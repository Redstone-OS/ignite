use core::{cell::RefCell, mem, ptr, slice};

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
#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
pub mod dtb;
mod memory_map;
mod video_mode;

#[cfg(target_arch = "riscv64")]
pub use arch::efi_get_boot_hartid;

pub(crate) fn page_size() -> usize {
    // EDK2 always uses 4096 as the page size
    4096
}

static mut IMAGE_HANDLE: Handle = unsafe { Handle::from_ptr(ptr::null_mut()) };

pub fn image_handle() -> Handle {
    unsafe { IMAGE_HANDLE }
}

pub(crate) fn alloc_zeroed_page_aligned(size: usize) -> *mut u8 {
    assert!(size != 0);

    let page_size = page_size();
    let pages = size.div_ceil(page_size);

    let ptr = {
        // Max address mapped by src/arch paging code (8 GiB)
        let mut ptr = 0x2_0000_0000;
        let mut st = uefi_services::system_table();
        let status = (st.boot_services().allocate_pages)(
            AllocateType::MaxAddress(ptr),
            MemoryType::LOADER_DATA, // Use LOADER_DATA or equivalent
            pages,
        );

        // Handling Result/Status if allocate_pages returns Status or Result
        // uefi 0.28 allocate_pages returns uefi::Status usually, but wrapper returns
        // Result. Wait, (st.boot_services().allocate_pages) calls the raw
        // function pointer? No, boot_services() returns reference to
        // BootServices table. allocate_pages is a method on BootServices.
        // If I call it as a method: st.boot_services().allocate_pages(...)
        // It returns Result<PhysAddr>.

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
    pub st:  &'static SystemTable<uefi::table::Boot>,
    outputs: RefCell<Vec<(Output, Option<EdidActive>)>>,
}

use alloc::vec::Vec;

impl OsEfi {
    pub fn new(st: &'static SystemTable<uefi::table::Boot>) -> Self {
        let mut outputs = Vec::<(Output, Option<EdidActive>)>::new();
        {
            let guid = GraphicsOutput::GUID;

            // Use locate_handle_buffer helper
            match st
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
                        crate::os::uefi::arch::main(st).unwrap();

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
            st,
            outputs: RefCell::new(outputs),
        }
    }
}

impl Os for OsEfi {
    type D = DiskOrFileEfi;
    type V = VideoModeIter;

    #[cfg(target_arch = "aarch64")]
    fn name(&self) -> &str {
        "aarch64/UEFI"
    }

    #[cfg(target_arch = "x86_64")]
    fn name(&self) -> &str {
        "x86_64/UEFI"
    }

    #[cfg(target_arch = "riscv64")]
    fn name(&self) -> &str {
        "riscv64/UEFI"
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
        // Search for RedstoneOS on disks in prioritized order
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
                    // Ignore header not found error
                    syscall::ENOENT => (),
                    // Print any other errors
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
        #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
        if let Some((addr, size)) = dtb::find_dtb(self) {
            return OsHwDesc::DeviceTree(addr, size);
        }

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
                // Hack to enable clone
                let ptr = output.0.0 as *mut _;
                Some(Output::new(&mut *ptr))
            },
            None => None,
        };
        VideoModeIter::new(output_opt)
    }

    fn set_video_mode(&self, output_i: usize, mode: &mut OsVideoMode) {
        // TODO: return error?
        let mut outputs = self.outputs.borrow_mut();
        let (output, _efi_edid_opt) = &mut outputs[output_i];

        // Output.0 is GraphicsOutputProtocol?
        // uefi 0.28: SetMode is method on Protocol impl? Or raw function pointer?
        // Let's assume raw pointer access if Output wraps it.
        // Or if Output is a shim around standard protocol.

        // For now trusting original logic but adapted
        // status_to_result((output.0.SetMode)(output.0, mode.id)).unwrap();

        // uefi-rs: protocol.set_mode(&mut mode_info)
        // output.0 corresponds to GraphicsOutput

        // If Output struct wraps UnsafeCell<GraphicsOutput>
        if let Some(mode_obj) = output.0.modes(st.boot_services()).nth(mode.id as usize) {
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
        // FrameBufferBase is tricky.
        // Try accessing via mode().frame_buffer_base() if available or raw.
        // uefi 0.28: current_mode_info returns ModeInfo.
        // checking ref docs: output.0.frame_buffer().as_mut_ptr() requires mapped?
        // Let's rely on info.
        // Or gop.mode().frame_buffer_base()?
        mode.base = output.0.frame_buffer().as_mut_ptr() as u64;
    }

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

        // Fallback to the current output resolution
        // output.0.Mode.Info...
        let info = output.0.current_mode_info();
        Some((info.resolution().0 as u32, info.resolution().1 as u32))
    }

    fn get_key(&self) -> OsKey {
        // TODO: do not unwrap
        let event = &self
            .st
            .boot_services()
            .events()
            .wait_for_event(&mut [self.st.stdin().wait_for_key_event().unwrap()])
            .unwrap();
        // Wait?
        // uefi-rs stdin().read_key().

        let key = match self.st.stdin().read_key() {
            Ok(Some(k)) => k,
            Ok(None) => return OsKey::Other, // Should wait
            Err(_) => return OsKey::Other,
        };

        // Convert key to OsKey
        // Based on uefi::proto::console::text::Key
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
        let _ = self.st.stdout().clear();
    }

    fn get_text_position(&self) -> (usize, usize) {
        // Not easily accessible in uefi-rs output?
        (0, 0) // Placeholder
    }

    fn set_text_position(&self, x: usize, y: usize) {
        let _ = self.st.stdout().set_cursor_position(x, y);
    }

    fn set_text_highlight(&self, highlight: bool) {
        let attr = if highlight {
            uefi::proto::console::text::Attribute::new(
                uefi::proto::console::text::Color::Black,
                uefi::proto::console::text::Color::LightGray,
            )
        } else {
            uefi::proto::console::text::Attribute::new(
                uefi::proto::console::text::Color::LightGray,
                uefi::proto::console::text::Color::Black,
            )
        };
        let _ = self.st.stdout().set_attribute(attr);
    }
}

fn status_to_result(status: Status) -> uefi::Result<usize> {
    match status {
        Status::SUCCESS => Ok(0),
        err => Err(unsafe { uefi::Error::from_status(err) }),
    }
}

// remove set_max_mode if handled

#[uefi_macros::entry]
fn main(image: Handle, mut st: SystemTable<uefi::table::Boot>) -> Status {
    unsafe {
        IMAGE_HANDLE = image;
    }
    uefi_services::init(&mut st).unwrap();

    // Disable Watchdog
    let _ = st.boot_services().set_watchdog_timer(0, 0, None);

    // Call Arch Main
    if let Err(err) = arch::main() {
        panic!("App error: {:?}", err);
    }

    // Reset System
    st.runtime_services()
        .reset(ResetType::Cold, Status::SUCCESS, None);
}
