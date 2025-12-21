//! Redstone OS Bootloader (Ignite) - Entry Point

#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(alloc_error_handler)]

extern crate alloc;

use ignite::{
    config::{BootConfig, Protocol, loader::load_configuration},
    core::{
        handoff::{BootInfo, FramebufferInfo as HandoffFbInfo},
        logging,
    },
    fs::{FileSystem, UefiFileSystem},
    memory::{BumpAllocator, PageTableManager, UefiFrameAllocator},
    protos::load_any,
    recovery::Diagnostics,
    security::{SecurityPolicy, validate_and_measure},
    uefi::{self, Handle, SystemTable},
    ui::Menu,
    video,
};

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, system_table: *mut SystemTable) -> ! {
    unsafe {
        uefi::init(system_table, image_handle);
        ignite::arch::x86::init();
        logging::init();
    }

    ignite::println!("Ignite Bootloader Iniciando...");

    // Heap Init
    unsafe {
        let heap_size = ignite::core::config::memory::BOOTLOADER_HEAP_SIZE;
        let heap_start = uefi::system_table()
            .boot_services()
            .allocate_pool(uefi::table::boot::MemoryType::LoaderData, heap_size)
            .expect("FALHA FATAL: Nao foi possivel alocar Heap inicial");

        ALLOCATOR.init(heap_start as usize, heap_size);
    }
    ignite::println!("Heap inicializada.");

    // FS Init
    let bs = uefi::system_table().boot_services();

    // Configurar FS
    let loaded_image_ptr = bs
        .open_protocol(
            image_handle,
            &uefi::proto::loaded_image::LOADED_IMAGE_PROTOCOL_GUID,
            image_handle,
            Handle::null(),
            uefi::table::boot::OPEN_PROTOCOL_GET_PROTOCOL,
        )
        .expect("Falha ao abrir LoadedImage");

    let loaded_image =
        unsafe { &*(loaded_image_ptr as *mut uefi::proto::loaded_image::LoadedImageProtocol) };
    let device_handle = loaded_image.device_handle;

    let fs_proto_ptr = bs
        .open_protocol(
            device_handle,
            &uefi::proto::media::fs::SIMPLE_FILE_SYSTEM_PROTOCOL_GUID,
            image_handle,
            Handle::null(),
            uefi::table::boot::OPEN_PROTOCOL_GET_PROTOCOL,
        )
        .expect("Falha ao abrir SimpleFileSystem");

    let mut fs_proto_ref =
        unsafe { &mut *(fs_proto_ptr as *mut uefi::proto::media::fs::SimpleFileSystemProtocol) };
    let mut boot_fs = UefiFileSystem::new(fs_proto_ref);

    // Carregar Configuração
    let config = match load_configuration(&mut boot_fs) {
        Ok(cfg) => cfg,
        Err(e) => {
            ignite::println!("AVISO: Erro ao carregar config: {:?}. Usando padrao.", e);
            BootConfig::default()
        },
    };

    // Configurar Vídeo
    let (_gop, mut fb_info) = video::init_video(bs).expect("FALHA CRITICA: Video GOP");

    // UI Menu
    let selected_entry = if !config.quiet && config.timeout.unwrap_or(0) > 0 {
        let fb_ptr = fb_info.addr;
        let ui_fb_info = HandoffFbInfo {
            address: fb_info.addr,
            size:    fb_info.size,
            width:   fb_info.width,
            height:  fb_info.height,
            stride:  fb_info.stride,
            format:  fb_info.format as u32,
        };

        let mut menu = Menu::new(&config);
        unsafe { menu.run(fb_ptr, ui_fb_info) }
    } else {
        config
            .entries
            .get(config.default_entry_idx)
            .expect("Configuracao invalida: Default Entry index fora dos limites")
    };

    ignite::println!("Bootando: {}", selected_entry.name);

    // Diagnóstico
    let health = Diagnostics::check_entry(&mut boot_fs, selected_entry);
    if let ignite::recovery::diagnostics::HealthStatus::Critical(msg) = health {
        panic!("Diagnostico falhou: {}", msg);
    }

    // Carregar arquivo para RAM (seja Kernel ou EFI App)
    let mut root_dir = boot_fs.root().expect("Falha raiz FS");
    let mut kernel_file = root_dir
        .open_file(&selected_entry.path)
        .expect("Kernel nao encontrado");
    let kernel_data = ignite::fs::read_to_bytes(kernel_file.as_mut()).expect("Erro leitura Kernel");

    // Segurança
    let policy = SecurityPolicy::new(&config);
    if let Err(e) = validate_and_measure(&kernel_data, &selected_entry.name, &policy) {
        panic!("Violacao de Seguranca: {:?}", e);
    }

    // --- RAMIFICAÇÃO DE PROTOCOLO ---

    if selected_entry.protocol == Protocol::EfiChainload {
        ignite::println!("Executando EFI Chainload...");

        // Para chainload, usamos LoadImage passando o buffer da memória
        // Isso evita depender de DevicePaths complexos
        let mut child_handle = Handle::null();
        let status = unsafe {
            (bs.load_image_f)(
                0, // BootFromMemory
                image_handle,
                core::ptr::null_mut(), // DevicePath (opcional se buffer)
                kernel_data.as_ptr() as *mut core::ffi::c_void,
                kernel_data.len(),
                &mut child_handle,
            )
        };

        if status.is_error() {
            panic!("Falha ao carregar imagem EFI: {:?}", status);
        }

        // Iniciar a imagem
        let mut exit_data_size: usize = 0;
        let mut exit_data: *mut u16 = core::ptr::null_mut();

        let status =
            unsafe { (bs.start_image_f)(child_handle, &mut exit_data_size, &mut exit_data) };

        if status.is_error() {
            ignite::println!("Aplicacao EFI retornou erro: {:?}", status);
            // Se falhar, voltamos ao menu ou paramos
            loop {
                core::hint::spin_loop();
            }
        } else {
            ignite::println!("Aplicacao EFI finalizada. Reiniciando...");
            let rt = uefi::system_table().runtime_services();
            rt.reset_system(uefi::table::runtime::ResetType::Cold, uefi::Status::SUCCESS);
        }
    }

    // --- PROTOCOLOS DE KERNEL (Nativo/Linux) ---

    let mut frame_allocator = UefiFrameAllocator::new(bs);
    let mut page_table = PageTableManager::new(&mut frame_allocator).expect("Falha PageTables");

    let launch_info = load_any(
        &mut frame_allocator,
        &mut page_table,
        &kernel_data,
        selected_entry.cmdline.as_deref(),
        alloc::vec![],
    )
    .expect("Falha ao preparar Kernel (Protocol Error)");

    ignite::println!("Saindo do UEFI...");

    // Exit Boot Services
    let (map_key, _iter) = get_memory_map_key(bs);
    if bs
        .exit_boot_services(image_handle, map_key)
        .to_result()
        .is_err()
    {
        let (retry_key, _) = get_memory_map_key(bs);
        if bs
            .exit_boot_services(image_handle, retry_key)
            .to_result()
            .is_err()
        {
            loop {
                core::hint::spin_loop();
            }
        }
    }

    // Jump
    unsafe {
        jump_to_kernel(
            launch_info.entry_point,
            launch_info.stack_pointer.unwrap_or(0),
            launch_info.rdi,
            launch_info.rsi,
            launch_info.rdx,
            launch_info.rbx,
            page_table.pml4_addr(),
        );
    }
}

fn get_memory_map_key(
    bs: &ignite::uefi::BootServices,
) -> (
    usize,
    impl Iterator<Item = ignite::memory::region::PhysicalMemoryRegion>,
) {
    let mut map_size = 0;
    let mut map_key = 0;
    let mut descriptor_size = 0;
    let mut descriptor_version = 0;
    let _ = (bs.get_memory_map_f)(
        &mut map_size,
        core::ptr::null_mut(),
        &mut map_key,
        &mut descriptor_size,
        &mut descriptor_version,
    );

    // Stub de retorno
    (map_key, core::iter::empty())
}

#[no_mangle]
unsafe extern "C" fn jump_to_kernel(
    entry: u64,
    stack: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    cr3: u64,
) -> ! {
    core::arch::asm!(
        "cli",
        "mov cr3, {cr3}",
        "cmp {stack}, 0",
        "je 2f",
        "mov rsp, {stack}",
        "mov rbp, 0",
        "2:",
        "mov rdi, {arg1}",
        "mov rsi, {arg2}",
        "mov rdx, {arg3}",
        "mov rcx, {arg4}",
        "mov rbx, {arg4}",
        "jmp {entry}",
        entry = in(reg) entry,
        stack = in(reg) stack,
        arg1 = in(reg) arg1,
        arg2 = in(reg) arg2,
        arg3 = in(reg) arg3,
        arg4 = in(reg) arg4,
        cr3 = in(reg) cr3,
        options(noreturn)
    );
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    ignite::panic_handler_impl(info);
}

#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    panic!("Out of Memory (OOM)");
}
