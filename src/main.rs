#![no_std]
#![cfg_attr(any(target_arch = "riscv64", target_os = "uefi"), no_main)]

//! # Redstone OS Bootloader (Ignite)
//!
//! Este arquivo contém o ponto de entrada principal e a lógica de inicialização
//! para o bootloader `ignite`. Ele é responsável por configurar o ambiente
//! inicial, carregar o kernel e o sistema de arquivos inicial (initfs),
//! configurar o layout de memória e passar o controle para o kernel.

mod redstonefs;

extern crate alloc;

// Panic handler para no_std com serial output detalhado
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        let port: u16 = 0x3F8;

        // Enviar "PANIC: "
        for &byte in b"PANIC: " {
            core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
        }

        // Tentar mostrar localização
        if let Some(location) = info.location() {
            // Arquivo
            for &byte in location.file().as_bytes() {
                core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
            }
            core::arch::asm!("out dx, al", in("dx") port, in("al") b':');

            // Linha (simplificado - apenas mostrar alguns dígitos)
            let line = location.line();
            if line >= 100 {
                let d = (line / 100) as u8;
                core::arch::asm!("out dx, al", in("dx") port, in("al") b'0' + d);
            }
            if line >= 10 {
                let d = ((line / 10) % 10) as u8;
                core::arch::asm!("out dx, al", in("dx") port, in("al") b'0' + d);
            }
            let d = (line % 10) as u8;
            core::arch::asm!("out dx, al", in("dx") port, in("al") b'0' + d);
        }

        // Nova linha
        for &byte in b"\r\n" {
            core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
        }
    }
    loop {}
}

use alloc::{boxed::Box, format, string::String, vec::Vec};
use core::{
    cmp,
    fmt::{self, Write},
    mem, ptr, slice, str,
};

use redstonefs::Disk;
use uefi::{print, println}; // Importar macros de I/O

use self::{
    arch::{paging_create, paging_framebuffer},
    os::{Os, OsHwDesc, OsKey, OsMemoryEntry, OsMemoryKind, OsVideoMode},
};

#[macro_use]
mod os;

mod arch;
mod io;
mod logger;
mod serial_16550;

const KIBI: usize = 1024;
const MIBI: usize = KIBI * KIBI;

// TODO: alocar isso de uma maneira mais razoável
static mut AREAS: [OsMemoryEntry; 1024] = [OsMemoryEntry {
    base: 0,
    size: 0,
    kind: OsMemoryKind::Null,
}; 1024];
static mut AREAS_LEN: usize = 0;

pub fn area_add(area: OsMemoryEntry) {
    #[allow(static_mut_refs)]
    unsafe {
        for existing_area in &mut AREAS[0..AREAS_LEN] {
            if existing_area.kind == area.kind {
                if existing_area.base.unchecked_add(existing_area.size) == area.base {
                    existing_area.size += area.size;
                    return;
                }
                if area.base.unchecked_add(area.size) == existing_area.base {
                    existing_area.size += area.size;
                    existing_area.base = area.base;
                    return;
                }
            }
        }
        *AREAS.get_mut(AREAS_LEN).expect("AREAS overflowed!") = area;
        AREAS_LEN += 1;
    }
}

pub static mut KERNEL_64BIT: bool = false;

pub static mut LIVE_OPT: Option<(u64, &'static [u8])> = None;

struct SliceWriter<'a> {
    slice: &'a mut [u8],
    i:     usize,
}

impl<'a> Write for SliceWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if let Some(slice_b) = self.slice.get_mut(self.i) {
                *slice_b = b;
                self.i += 1;
            } else {
                return Err(fmt::Error);
            }
        }
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
#[repr(C, packed(8))]
/// Argumentos passados para o kernel durante a inicialização.
pub struct KernelArgs {
    kernel_base: u64,
    kernel_size: u64,
    stack_base:  u64,
    stack_size:  u64,
    env_base:    u64,
    env_size:    u64,

    /// O ponteiro base para o RSDP salvo.
    ///
    /// Este campo pode ser NULL e, se for, o sistema não inicializou com UEFI
    /// ou de alguma outra forma não recuperou os RSDPs. O kernel ou um driver
    /// de espaço de usuário tentará, portanto, pesquisar a memória BIOS. Em
    /// sistemas UEFI, a pesquisa não é garantida de funcionar.
    acpi_rsdp_base: u64,
    /// O tamanho da região RSDP.
    acpi_rsdp_size: u64,

    areas_base: u64,
    areas_size: u64,

    bootstrap_base: u64,
    bootstrap_size: u64,
}

fn select_mode(os: &impl Os, output_i: usize, live: &mut bool) -> Option<OsVideoMode> {
    let mut modes = Vec::new();
    for mode in os.video_modes(output_i) {
        modes.push(mode);
    }

    if modes.is_empty() {
        return None;
    }

    // Ordenar modos por área de pixel, maior primeiro
    modes.sort_by(|a, b| (b.width * b.height).cmp(&(a.width * a.height)));

    // Selecionar automaticamente a melhor resolução
    print!("Saida {}", output_i);
    let mut selected_mode = modes.first().copied();

    if let Some((best_width, best_height)) = os.best_resolution(output_i) {
        print!(", melhor resolucao: {}x{}", best_width, best_height);
        // Tentar encontrar a resolução exata recomendada
        for mode in modes.iter() {
            if mode.width == best_width && mode.height == best_height {
                selected_mode = Some(*mode);
                break;
            }
        }
    }

    if let Some(mode) = selected_mode {
        println!(" -> Selecionado: {}x{}", mode.width, mode.height);
    } else {
        println!();
    }

    selected_mode
}

fn redstonefs<O: Os>(os: &O) -> (redstonefs::FileSystem<O::D>, Option<&'static [u8]>) {
    let attempts = 10;
    for attempt in 0..=attempts {
        let mut password_opt = None;
        if attempt > 0 {
            print!("\rRedstoneFS password ({}/{}): ", attempt, attempts);

            let mut password = String::new();

            loop {
                match os.get_key() {
                    OsKey::Backspace | OsKey::Delete => {
                        if !password.is_empty() {
                            print!("\x08 \x08");
                            password.pop();
                        }
                    },
                    OsKey::Char(c) => {
                        print!("*");
                        password.push(c)
                    },
                    OsKey::Enter => break,
                    _ => (),
                }
            }

            // Apagar informações da senha
            while os.get_text_position().0 > 0 {
                print!("\x08 \x08");
            }

            if !password.is_empty() {
                password_opt = Some(password);
            }
        }
        match os.filesystem(password_opt.as_ref().map(|x| x.as_bytes())) {
            Ok(fs) => {
                return (
                    fs,
                    password_opt.map(|password| {
                        // Copiar senha para memória alinhada por página
                        let password_size = password.len();
                        let password_base = os.alloc_zeroed_page_aligned(password_size);

                        area_add(OsMemoryEntry {
                            base: password_base as u64,
                            size: password_size as u64,
                            kind: OsMemoryKind::Reserved,
                        });

                        unsafe {
                            ptr::copy(password.as_ptr(), password_base, password_size);
                            slice::from_raw_parts(password_base, password_size)
                        }
                    }),
                );
            },
            Err(err) => match err.errno {
                // Senha incorreta, tente novamente
                syscall::ENOKEY => (),
                _ => {
                    panic!("Failed to open RedstoneFS: {:?}", err);
                },
            },
        }
    }
    panic!("RedstoneFS: Numero maximo de tentativas excedido");
}

#[derive(PartialEq)]
enum Filetype {
    Elf,
    Initfs,
}
fn load_to_memory<O: Os>(
    os: &O,
    fs: &mut redstonefs::FileSystem<O::D>,
    dirname: &str,
    filename: &str,
    filetype: Filetype,
) -> &'static mut [u8] {
    fs.tx(|tx| {
        let dir_node = tx
            .find_node(redstonefs::TreePtr::root(), dirname)
            .unwrap_or_else(|err| panic!("Failed to find {} directory: {:?}", dirname, err));

        let node = tx
            .find_node(dir_node.ptr(), filename)
            .unwrap_or_else(|err| panic!("Failed to find {} file: {:?}", filename, err));

        let size = node.data().size();

        print!("{}: 0/{} MiB", filename, size / MIBI as u64);

        let ptr = os.alloc_zeroed_page_aligned(size as usize);
        if ptr.is_null() {
            panic!("Failed to allocate memory for {}", filename);
        }

        let slice = unsafe { slice::from_raw_parts_mut(ptr, size as usize) };

        let mut i = 0;
        for chunk in slice.chunks_mut(MIBI) {
            print!(
                "\r{}: {}/{} MiB",
                filename,
                i / MIBI as u64,
                size / MIBI as u64
            );
            i += tx
                .read_node_inner(&node, i, chunk)
                .unwrap_or_else(|err| panic!("Failed to read `{}` file: {:?}", filename, err))
                as u64;
        }
        println!(
            "\r{}: {}/{} MiB",
            filename,
            i / MIBI as u64,
            size / MIBI as u64
        );

        if filetype == Filetype::Elf {
            let magic = &slice[..4];
            if magic != b"\x7FELF" {
                panic!("{} has invalid magic number {:#X?}", filename, magic);
            }
        } else if filetype == Filetype::Initfs {
            let magic = &slice[..8];
            if magic != b"RedstoneFtw" {
                panic!("{} has invalid magic number {:#X?}", filename, magic);
            }
        }

        Ok(slice)
    })
    .unwrap_or_else(|err| {
        panic!(
            "Falha na transacao RedstoneFS ao carregar `{}`: {:?}",
            filename, err
        )
    })
}

fn elf_entry(data: &[u8]) -> (u64, bool) {
    match (data[4], data[5]) {
        // 32-bit, little endian
        (1, 1) => (
            u32::from_le_bytes(
                <[u8; 4]>::try_from(&data[0x18..0x18 + 4]).expect("conversion cannot fail"),
            ) as u64,
            false,
        ),
        // 32-bit, big endian
        (1, 2) => (
            u32::from_be_bytes(
                <[u8; 4]>::try_from(&data[0x18..0x18 + 4]).expect("conversion cannot fail"),
            ) as u64,
            false,
        ),
        // 64-bit, little endian
        (2, 1) => (
            u64::from_le_bytes(
                <[u8; 8]>::try_from(&data[0x18..0x18 + 8]).expect("conversion cannot fail"),
            ),
            true,
        ),
        // 64-bit, big endian
        (2, 2) => (
            u64::from_be_bytes(
                <[u8; 8]>::try_from(&data[0x18..0x18 + 8]).expect("conversion cannot fail"),
            ),
            true,
        ),
        (ei_class, ei_data) => {
            panic!(
                "ELF EI_CLASS {} EI_DATA {} nao suportado",
                ei_class, ei_data
            );
        },
    }
}

fn ignite_main(os: &impl Os) -> (usize, u64, KernelArgs) {
    println!(
        "Bootloader Redstone OS {} em {}",
        env!("CARGO_PKG_VERSION"),
        os.name()
    );

    let hwdesc = os.hwdesc();
    println!("Descritor de Hardware: {:x?}", hwdesc);
    let (acpi_rsdp_base, acpi_rsdp_size) = match hwdesc {
        OsHwDesc::Acpi(base, size) => (base, size),
        OsHwDesc::DeviceTree(base, size) => (base, size),
        OsHwDesc::NotFound => (0, 0),
    };

    // TODO(RFS): Reabilitar RedstoneFS quando a biblioteca `libs/rfs` estiver
    // pronta Por enquanto, usamos carregamento direto da ESP via UEFI
    // let (mut fs, password_opt) = redstonefs(os);

    // print!("RedstoneFS ");
    // for i in 0..fs.header.uuid().len() {
    //     if i == 4 || i == 6 || i == 8 || i == 10 {
    //         print!("-");
    //     }

    //     print!("{:>02x}", fs.header.uuid()[i]);
    // }
    // println!(": {} MiB", fs.header.size() / MIBI as u64);
    println!("Sistema de Arquivos: UEFI (FAT32) - Solucao Temporaria");
    println!();

    let mut mode_opts = Vec::new();
    let mut live = cfg!(feature = "live");
    for output_i in 0..os.video_outputs() {
        if output_i > 0 {
            os.clear_text();
        }
        mode_opts.push(select_mode(os, output_i, &mut live));
    }

    let stack_size = 128 * KIBI;
    let stack_base = os.alloc_zeroed_page_aligned(stack_size);
    if stack_base.is_null() {
        panic!("Falha ao alocar memoria para a pilha");
    }

    // Live disk functionality disabled for UEFI boot workaround
    let live_opt: Option<&'static mut [u8]> = None;

    let (kernel, kernel_entry) = {
        // let kernel = load_to_memory(os, &mut fs, "boot", "kernel", Filetype::Elf);
        let path = "boot/kernel";
        print!("Carregando {}: ", path);
        let kernel_vec = os.read_file(path).expect("Falha ao encontrar boot/kernel");
        let kernel_size = kernel_vec.len();
        println!("{} MiB ({} bytes)", kernel_size / MIBI, kernel_size);

        if kernel_size == 0 {
            panic!("ERRO: Kernel tem tamanho zero!");
        }

        // We need to keep this memory allocated. Vec usually allocates on heap.
        // We leak the vec to get a slice with static lifetime conceptually for the boot
        // process Ideally we should copy to page aligned memory if Vec is not,
        // but os.alloc_zeroed uses UEFI allocator which returns aligned memory?
        // Vec uses Global allocator. For simplicity, we just leak it into a
        // slice.
        let kernel_slice = Box::leak(kernel_vec.into_boxed_slice());

        println!("[DEBUG] Kernel carregado, analisando ELF...");

        // Verificar magic number ELF
        if kernel_slice.len() < 4 {
            panic!("Kernel muito pequeno para ser um ELF valido!");
        }
        if &kernel_slice[0..4] != b"\x7FELF" {
            panic!(
                "Kernel nao tem magic number ELF valido: {:X?}",
                &kernel_slice[0..4]
            );
        }

        let (kernel_entry_offset, kernel_64bit) = elf_entry(kernel_slice);

        // CRITICAL: O entry point no ELF é um offset relativo ao início do arquivo.
        // Precisamos adicionar o endereço base onde o kernel foi carregado.
        let kernel_base_addr = kernel_slice.as_ptr() as u64;
        let kernel_entry_absolute = kernel_base_addr + kernel_entry_offset;

        println!("[DEBUG] Kernel carregado em: 0x{:X}", kernel_base_addr);
        println!("[DEBUG] Entry offset (ELF): 0x{:X}", kernel_entry_offset);
        println!(
            "[DEBUG] Entry absoluto: 0x{:X}, 64-bit: {}",
            kernel_entry_absolute, kernel_64bit
        );
        unsafe {
            KERNEL_64BIT = kernel_64bit;
        }
        (kernel_slice, kernel_entry_absolute)
    };

    let (bootstrap_size, bootstrap_base) = {
        // let initfs_slice = load_to_memory(os, &mut fs, "boot", "initfs",
        // Filetype::Initfs);
        let path = "boot/initfs";
        print!("Carregando {}: ", path);
        let initfs_vec = os.read_file(path).expect("Falha ao encontrar boot/initfs");
        let initfs_size = initfs_vec.len();
        println!("{} MiB ({} bytes)", initfs_size / MIBI, initfs_size);

        if initfs_size == 0 {
            panic!("ERRO: InitFS tem tamanho zero!");
        }

        let initfs_slice = Box::leak(initfs_vec.into_boxed_slice());

        // let magic = &initfs_slice[..8];
        // if magic != b"RedstoneFtw" {
        //     panic!("{} has invalid magic number {:#X?}", path, magic);
        // }

        let memory = unsafe {
            let total_size = initfs_slice.len().next_multiple_of(4096);
            let ptr = os.alloc_zeroed_page_aligned(total_size);
            assert!(
                !ptr.is_null(),
                "falha ao alocar memoria para bootstrap+initfs"
            );
            core::slice::from_raw_parts_mut(ptr, total_size)
        };
        memory[..initfs_slice.len()].copy_from_slice(initfs_slice);

        (memory.len() as u64, memory.as_mut_ptr() as u64)
    };

    println!("[DEBUG] Configurando paginacao...");
    let page_phys = unsafe { paging_create(os, kernel.as_ptr() as u64, kernel.len() as u64) }
        .expect("Falha ao configurar paginacao");
    println!("[DEBUG] Paginacao configurada em: 0x{:X}", page_phys);

    let mut env_size = 64 * KIBI;
    let env_base = os.alloc_zeroed_page_aligned(env_size);
    if env_base.is_null() {
        panic!("Falha ao alocar memoria para a pilha de ambiente");
    }

    {
        let mut w = SliceWriter {
            slice: unsafe { slice::from_raw_parts_mut(env_base, env_size) },
            i:     0,
        };

        match hwdesc {
            OsHwDesc::Acpi(addr, size) => {
                writeln!(w, "RSDP_ADDR={addr:016x}").unwrap();
                writeln!(w, "RSDP_SIZE={size:016x}").unwrap();
            },
            OsHwDesc::DeviceTree(addr, size) => {
                writeln!(w, "DTB_ADDR={addr:016x}").unwrap();
                writeln!(w, "DTB_SIZE={size:016x}").unwrap();
            },
            OsHwDesc::NotFound => {},
        }

        if let Some(live) = live_opt {
            writeln!(w, "DISK_LIVE_ADDR={:016x}", live.as_ptr() as usize).unwrap();
            writeln!(w, "DISK_LIVE_SIZE={:016x}", live.len()).unwrap();
            writeln!(w, "RESTONEFS_BLOCK={:016x}", 0).unwrap();
        } else {
            // Placeholder for FAT32 workaround
            writeln!(w, "RESTONEFS_BLOCK={:016x}", 0).unwrap();
        }
        write!(w, "RESTONEFS_UUID=").unwrap();
        // for i in 0..fs.header.uuid().len() {
        //     if i == 4 || i == 6 || i == 8 || i == 10 {
        //         write!(w, "-").unwrap();
        //     }

        //     write!(w, "{:>02x}", fs.header.uuid()[i]).unwrap();
        // }
        // Fake UUID for now
        write!(w, "00000000-0000-0000-0000-000000000000").unwrap();
        writeln!(w).unwrap();
        // if let Some(password) = password_opt {
        //     writeln!(
        //         w,
        //         "RESTONEFS_PASSWORD_ADDR={:016x}",
        //         password.as_ptr() as usize
        //     )
        //     .unwrap();
        //     writeln!(w, "RESTONEFS_PASSWORD_SIZE={:016x}", password.len()).unwrap();
        // }

        #[cfg(target_arch = "riscv64")]
        {
            let boot_hartid = os::efi_get_boot_hartid()
                .expect("Falha ao recuperar boot hart id da implementacao EFI!");
            writeln!(w, "BOOT_HART_ID={:016x}", boot_hartid).unwrap();
        }

        for output_i in 0..os.video_outputs() {
            if let Some(mut mode) = mode_opts[output_i] {
                // Definir modo para obter valores atualizados
                os.set_video_mode(output_i, &mut mode);

                if output_i == 0 {
                    let virt = unsafe {
                        paging_framebuffer(
                            os,
                            page_phys,
                            mode.base,
                            (mode.stride * mode.height * 4) as u64,
                        )
                    }
                    .expect("Falha ao mapear framebuffer");

                    writeln!(w, "FRAMEBUFFER_ADDR={:016x}", mode.base).unwrap();
                    writeln!(w, "FRAMEBUFFER_VIRT={virt:016x}").unwrap();
                    writeln!(w, "FRAMEBUFFER_WIDTH={:016x}", mode.width).unwrap();
                    writeln!(w, "FRAMEBUFFER_HEIGHT={:016x}", mode.height).unwrap();
                    writeln!(w, "FRAMEBUFFER_STRIDE={:016x}", mode.stride).unwrap();
                } else {
                    writeln!(
                        w,
                        "FRAMEBUFFER{}={:#x},{},{},{}",
                        output_i, mode.base, mode.width, mode.height, mode.stride,
                    )
                    .unwrap();
                }
            }
        }

        env_size = w.i;
    }

    println!("[DEBUG] Retornando controle para arch::main...");
    #[allow(static_mut_refs)]
    (
        page_phys,
        kernel_entry,
        KernelArgs {
            kernel_base: kernel.as_ptr() as u64,
            kernel_size: kernel.len() as u64,
            stack_base: stack_base as u64,
            stack_size: stack_size as u64,
            env_base: env_base as u64,
            env_size: env_size as u64,
            acpi_rsdp_base,
            acpi_rsdp_size,
            areas_base: unsafe { AREAS.as_ptr() as u64 },
            areas_size: unsafe { (AREAS.len() * mem::size_of::<OsMemoryEntry>()) as u64 },
            bootstrap_base,
            bootstrap_size,
        },
    )
}
