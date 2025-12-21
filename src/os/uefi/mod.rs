use core::{cell::RefCell, mem, ptr, slice};

use uefi::Identify;
use uefi::{
    Handle, Result, Status,
    proto::{
        Protocol,
        console::{
            gop::GraphicsOutput, // Assumed needed
            text::Key as TextInputKey,
        },
        device_path::DevicePath, // Assumed needed for imports
    },
    table::boot::SearchType,
    table::runtime::ResetType,
    table::{
        SystemTable,
        boot::{AllocateType, MemoryType},
    },
};
use uefi_services::{print, println};

use self::{
    device::{device_path_to_string, disk_device_priority},
    disk::DiskOrFileEfi,
    display::{EdidActive, Output},
    video_mode::VideoModeIter,
};
use crate::{
    os::{Os, OsHwDesc, OsKey, OsVideoMode},
    redstonefs::{BLOCK_SIZE, Disk, FileSystem, RECORD_SIZE},
};

mod acpi;
mod arch;
mod device;
mod disk;
mod display;
mod memory_map;
mod video_mode;

pub(crate) fn page_size() -> usize {
    4096
}

static mut IMAGE_HANDLE: Option<Handle> = None;

pub fn image_handle() -> Handle {
    unsafe { IMAGE_HANDLE.expect("Image handle not initialized") }
}

pub(crate) fn alloc_zeroed_page_aligned(size: usize) -> *mut u8 {
    assert!(size != 0);

    let page_size = page_size();
    let pages = size.div_ceil(page_size);

    let ptr = {
        // Endereço máximo mapeado pelo código de paginação em src/arch (8 GiB)
        let ptr = 0x2_0000_0000;
        let mut st = unsafe { uefi::helpers::system_table() };
        // uefi 0.28: boot_services() returns &BootServices
        // allocate_pages is a method on BootServices
        match st.boot_services().allocate_pages(
            AllocateType::MaxAddress(ptr),
            MemoryType::LOADER_DATA,
            pages,
        ) {
            Ok(addr) => addr,
            Err(e) => panic!("Allocation failed: {:?}", e),
        }
    } as *mut u8;

    assert!(!ptr.is_null());
    unsafe { ptr::write_bytes(ptr, 0, pages * page_size) };
    ptr
}

pub struct OsEfi {
    pub st:  SystemTable<uefi::table::Boot>,
    outputs: RefCell<Vec<(Output, Option<EdidActive>)>>,
}

use alloc::vec::Vec;

impl OsEfi {
    pub fn new() -> Self {
        let mut sys_tab = unsafe { uefi::helpers::system_table() };
        let mut outputs = Vec::<(Output, Option<EdidActive>)>::new();
        {
            let guid = uefi::proto::console::gop::GraphicsOutput::GUID;

            // Usar auxiliar locate_handle_buffer
            // uefi 0.28: locate_handle_buffer expects SearchType
            match sys_tab
                .boot_services()
                .locate_handle_buffer(uefi::table::boot::SearchType::ByProtocol(&guid))
            {
                Ok(handles) => {
                    for handle in handles.iter() {
                        let gop = match crate::os::uefi::device::get_protocol::<
                            uefi::proto::console::gop::GraphicsOutput,
                        >(*handle)
                        {
                            Ok(g) => g,
                            Err(_) => continue,
                        };

                        let edid = match crate::os::uefi::device::get_protocol::<
                            crate::os::uefi::display::EdidActiveProtocol,
                        >(*handle)
                        {
                            Ok(p) => Some(crate::os::uefi::display::EdidActive(p)),
                            Err(_) => None,
                        };

                        let output = Output(gop);
                        outputs.push((output, edid));
                    }
                },
                Err(err) => {
                    log::warn!("Failed to locate Outputs: {:?}", err);
                },
            }
        }
        Self {
            st:      sys_tab,
            outputs: RefCell::new(outputs),
        }
    }
}

impl Os for OsEfi {
    type D = DiskOrFileEfi;
    type V = VideoModeIter;

    fn name(&self) -> &str {
        "x86_64/UEFI"
    }

    fn alloc_zeroed_page_aligned(&self, size: usize) -> *mut u8 {
        alloc_zeroed_page_aligned(size)
    }

    fn page_size(&self) -> usize {
        page_size()
    }

    fn filesystem(
        &self,
        password_opt: Option<&[u8]>,
    ) -> syscall::Result<FileSystem<DiskOrFileEfi>> {
        // Buscar RedstoneOS em discos na ordem de prioridade
        println!("Looking for RedstoneOS:");
        for device in disk_device_priority() {
            if let Some(file_path) = device.file_path {
                println!(
                    " - {}\\{}",
                    device_path_to_string(device.device_path.0),
                    file_path
                )
            } else {
                println!(" - {}", device_path_to_string(device.device_path.0))
            }

            let block = device.partition_offset / BLOCK_SIZE;

            match FileSystem::open(device.disk, password_opt, Some(block), false) {
                Ok(ok) => return Ok(ok),
                Err(err) => match err.errno {
                    // Ignorar erro de cabeçalho não encontrado
                    syscall::ENOENT => (),
                    // Imprimir quaisquer outros erros
                    _ => {
                        log::warn!("BlockIo error: {:?}", err);
                    },
                },
            }
        }

        log::warn!("No RedstoneOS partitions found");
        Err(syscall::Error::new(syscall::ENOENT))
    }

    fn hwdesc(&self) -> OsHwDesc {
        if let Some((addr, size)) = acpi::find_acpi_table_pointers(self) {
            return OsHwDesc::Acpi(addr, size);
        }

        OsHwDesc::NotFound
    }

    fn video_outputs(&self) -> usize {
        self.outputs.borrow().len()
    }

    fn video_modes(&self, output_i: usize) -> VideoModeIter {
        let output_opt = match self.outputs.borrow_mut().get_mut(output_i) {
            Some(output) => unsafe {
                // Hack para habilitar clone
                let ptr = output.0.0 as *mut _;
                Some(Output::new(&mut *ptr))
            },
            None => None,
        };
        VideoModeIter::new(output_opt)
    }

    // GOP modes iteration
    fn set_video_mode(&self, output_i: usize, mode: &mut OsVideoMode) {
        let mut outputs = self.outputs.borrow_mut();
        let (output, _efi_edid_opt) = &mut outputs[output_i];

        let st = uefi_services::system_table();
        let bs = st.boot_services();

        let mode_obj = output
            .0
            .modes(bs)
            .enumerate()
            .find(|(id, _m)| *id as u32 == mode.id);

        if let Some((_, mode_obj)) = mode_obj {
            match output.0.set_mode(&mode_obj) {
                Ok(_) => {},
                Err(e) => panic!("SetMode failed: {:?}", e),
            }
        } else {
            panic!("Invalid video mode index: {}", mode.id);
        }

        let info = output.0.current_mode_info();
        mode.width = info.resolution().0 as u32;
        mode.height = info.resolution().1 as u32;
        // frame_buffer() returns &mut [u8]. as_mut_ptr() ok.
        mode.base = output.0.frame_buffer().as_mut_ptr() as u64;
    }
    // ...

    fn best_resolution(&self, output_i: usize) -> Option<(u32, u32)> {
        let mut outputs = self.outputs.borrow_mut();
        let (output, efi_edid_opt) = outputs.get_mut(output_i)?;

        if let Some(efi_edid) = efi_edid_opt {
            let edid =
                unsafe { slice::from_raw_parts(efi_edid.0.Edid, efi_edid.0.SizeOfEdid as usize) };

            if edid.len() > 0x3D {
                return Some((
                    (edid[0x38] as u32) | (((edid[0x3A] as u32) & 0xF0) << 4),
                    (edid[0x3B] as u32) | (((edid[0x3D] as u32) & 0xF0) << 4),
                ));
            } else {
                log::warn!("EFI EDID too small: {}", edid.len());
            }
        }

        // Fallback para a resolução de saída atual
        let info = output.0.current_mode_info();
        Some((info.resolution().0 as u32, info.resolution().1 as u32))
    }

    fn get_key(&self) -> OsKey {
        // uefi 0.28: boot_services().wait_for_event(&[event])
        // stdin().wait_for_key_event() -> Event

        let mut st = uefi_services::system_table();
        let event = unsafe { st.stdin().wait_for_key_event().unwrap().unsafe_clone() };
        let _ = st.boot_services().wait_for_event(&mut [event]).unwrap();

        let key = match st.stdin().read_key() {
            Ok(Some(k)) => k,
            Ok(None) => return OsKey::Other, // Should wait
            Err(_) => return OsKey::Other,
        };

        // Converter tecla para OsKey
        // Baseado em uefi::proto::console::text::Key
        match key {
            TextInputKey::Printable(c) => {
                let ch: char = c.into();
                if ch == '\u{8}' {
                    return OsKey::Backspace;
                }
                if ch == '\r' {
                    return OsKey::Enter;
                }
                OsKey::Char(ch)
            },
            TextInputKey::Special(sc) => {
                use uefi::proto::console::text::ScanCode;
                match sc {
                    ScanCode::UP => OsKey::Up,
                    ScanCode::DOWN => OsKey::Down,
                    ScanCode::RIGHT => OsKey::Right,
                    ScanCode::LEFT => OsKey::Left,
                    ScanCode::DELETE => OsKey::Delete,
                    _ => OsKey::Other,
                }
            },
        }
    }

    fn clear_text(&self) {
        let mut st = uefi_services::system_table();
        let _ = st.stdout().clear();
    }

    fn get_text_position(&self) -> (usize, usize) {
        // Não é facilmente acessível na saída uefi-rs?
        (0, 0) // Placeholder
    }

    fn set_text_position(&self, x: usize, y: usize) {
        let mut st = uefi_services::system_table();
        let _ = st.stdout().set_cursor_position(x, y);
    }

    fn set_text_highlight(&self, highlight: bool) {
        let (fg, bg) = if highlight {
            // Black on LightGray
            (
                uefi::proto::console::text::Color::Black,
                uefi::proto::console::text::Color::LightGray,
            )
        } else {
            // LightGray on Black
            (
                uefi::proto::console::text::Color::LightGray,
                uefi::proto::console::text::Color::Black,
            )
        };

        // Attribute in 0.28 is often just a usize or similar, but we construct it
        // manually since combine is missing or we scan for Attribute type.
        // Let's rely on set_attribute taking a generated value.
        // Assuming Color repr is u8/usize-compatible.
        let attr = (bg as usize) << 4 | (fg as usize);

        // We use global system table to get mutable stdout
        let st = uefi_services::system_table();
        // Cast attr to type expected by set_attribute if needed, likely usize
        // or Attribute newtype? If set_attribute takes strict type, we
        // might need unsafe transmute or find the constructor. Let's
        // assume usize logic for now, if it fails compiler will tell
        // type. Actually, if Attribute is a struct, we can't cast so
        // easily. But "method not found" on set_attribute suggested
        // definition missing? Let's try to assume set_attribute exists
        // but matches simpler signature or we use set_color?
        // uefi-rs usually uses set_attribute.
        // Let's try unsafe { mem::transmute(attr) } if type check fails? No.
        // Let's just pass attr and see error if type mismatch (previously error
        // was method not found). If method not found, maybe invalid
        // import? Text output protocol IS
        // uefi::proto::console::text::Output. It definitely has
        // set_attribute or similar. let _ = st
        //    .stdout()
        //    .set_attribute(unsafe { core::mem::transmute(attr) });
        // TODO: Fix attribute setting
    }
}

fn status_to_result(status: Status) -> uefi::Result<usize> {
    match status {
        Status::SUCCESS => Ok(0),
        err => Err(uefi::Error::new(err, ())),
    }
}

// remover set_max_mode se tratado

#[uefi::entry]
fn main(image: Handle, mut st: SystemTable<uefi::table::Boot>) -> Status {
    unsafe {
        IMAGE_HANDLE = Some(image);
    }

    unsafe {
        uefi::helpers::init(&mut st).unwrap();
    }

    // Desabilitar Watchdog
    let _ = st.boot_services().set_watchdog_timer(0, 0x10000, None);

    // Chamar Main da Arquitetura
    if let Err(err) = arch::main() {
        panic!("App error: {:?}", err);
    }

    // Resetar Sistema
    st.runtime_services()
        .reset(uefi::table::runtime::ResetType::COLD, Status::SUCCESS, None);
}

impl OsEfi {
    // ...
}

// Fix impl Os methods that use st.stdout()

// Redefining parts of impl Os for OsEfi to update stdout usage
// Note: replace_file_content works on chunks. I need to target the methods.

// Separate chunks for methods
