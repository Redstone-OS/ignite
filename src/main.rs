//! Redstone OS Bootloader (Ignite) - Entry Point
//!
//! O Executor Principal.
//! Responsável por orquestrar a inicialização do hardware, carregar a
//! configuração, interagir com o usuário e passar o controle para o Kernel.

#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(alloc_error_handler)] // Habilita o handler de OOM customizado

extern crate alloc;

// Imports da biblioteca Ignite
use ignite::{
    config::{BootConfig, Protocol, loader::load_configuration},
    core::{
        handoff::{BootInfo, FramebufferInfo as HandoffFbInfo}, // Alias para evitar colisão
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

// ============================================================================
// Alocador Global
// ============================================================================

// Define o alocador de memória para este binário.
#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

// ============================================================================
// Ponto de Entrada UEFI
// ============================================================================

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, system_table: *mut SystemTable) -> ! {
    // 1. Inicialização Básica (Sem Heap)
    unsafe {
        uefi::init(system_table, image_handle);
        ignite::arch::x86::init(); // Inicializa COM1
        logging::init(); // Conecta o Logger ao COM1
    }

    ignite::println!("Ignite Bootloader Iniciando...");

    // 2. Inicializar Heap Global
    unsafe {
        let heap_size = ignite::core::config::memory::BOOTLOADER_HEAP_SIZE;
        let heap_start = uefi::system_table()
            .boot_services()
            .allocate_pool(uefi::table::boot::MemoryType::LoaderData, heap_size)
            .expect("FALHA FATAL: Nao foi possivel alocar Heap inicial");

        ALLOCATOR.init(heap_start as usize, heap_size);
    }
    ignite::println!("Heap inicializada.");

    // 3. Configurar Sistema de Arquivos de Boot (ESP)
    let bs = uefi::system_table().boot_services();

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

    // 4. Carregar Configuração
    // Tenta ler do disco. Se falhar ou retornar config vazia, força Rescue.
    let mut config = match load_configuration(&mut boot_fs) {
        Ok(cfg) => cfg,
        Err(e) => {
            ignite::println!(
                "AVISO: Erro critico na config: {:?}. Entrando em modo Recovery.",
                e
            );
            BootConfig::recovery()
        },
    };

    // REDE DE SEGURANÇA: Se a config carregada não tiver entradas (ex: arquivo
    // vazio ou parser falhou silenciosamente), força o modo de recuperação para
    // evitar pânico na UI.
    if config.entries.is_empty() {
        ignite::println!(
            "AVISO: Nenhuma entrada encontrada na configuracao. Ativando modo Recovery."
        );
        config = BootConfig::recovery();
    }

    // 5. Configurar Vídeo (GOP)
    let (_gop, mut fb_info) =
        video::init_video(bs).expect("FALHA CRITICA: Nao foi possivel iniciar Video GOP");

    // 6. Interface de Usuário (Menu Gráfico)
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
        // Fallback seguro se o índice padrão for inválido
        if config.default_entry_idx >= config.entries.len() {
            &config.entries[0]
        } else {
            &config.entries[config.default_entry_idx]
        }
    };

    ignite::println!("Bootando: {}", selected_entry.name);

    // 7. Diagnóstico
    let health = Diagnostics::check_entry(&mut boot_fs, selected_entry);
    if let ignite::recovery::diagnostics::HealthStatus::Critical(msg) = health {
        panic!(
            "Diagnostico falhou para entrada '{}': {}",
            selected_entry.name, msg
        );
    }

    // 8. Carregar Kernel
    let mut root_dir = boot_fs.root().expect("Falha raiz FS");
    let mut kernel_file = root_dir
        .open_file(&selected_entry.path)
        .expect("Kernel nao encontrado no disco");
    let kernel_data =
        ignite::fs::read_to_bytes(kernel_file.as_mut()).expect("Erro de I/O ao ler Kernel");

    // 9. Segurança
    let policy = SecurityPolicy::new(&config);
    if let Err(e) = validate_and_measure(&kernel_data, &selected_entry.name, &policy) {
        panic!("Violacao de Seguranca detectada: {:?}", e);
    }

    // 10. Executar Protocolo de Boot
    // RAMIFICAÇÃO: Chainload vs Kernel Nativo

    if selected_entry.protocol == Protocol::EfiChainload {
        ignite::println!("Executando EFI Chainload...");

        let mut child_handle = Handle::null();

        // LoadImage espera SourceBuffer se BootPolicy=FALSE(0)
        let status = unsafe {
            (bs.load_image_f)(
                0, // Boot from Memory
                image_handle,
                core::ptr::null_mut(),
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

        // Passa o controle para o aplicativo EFI (Shell)
        let status =
            unsafe { (bs.start_image_f)(child_handle, &mut exit_data_size, &mut exit_data) };

        if status.is_error() {
            ignite::println!("Aplicacao EFI retornou erro: {:?}", status);
            // Se falhar, voltamos ao menu ou paramos
            loop {
                core::hint::spin_loop();
            }
        } else {
            // Se o app retornar (ex: usuário digitou 'exit' no shell), reinicia.
            ignite::println!("App finalizado. Reiniciando sistema...");
            let rt = uefi::system_table().runtime_services();
            rt.reset_system(uefi::table::runtime::ResetType::Cold, uefi::Status::SUCCESS);
        }
    }

    // --- CAMINHO KERNEL NATIVO / LINUX ---

    let mut frame_allocator = UefiFrameAllocator::new(bs);
    let mut page_table =
        PageTableManager::new(&mut frame_allocator).expect("Falha ao criar PageTables");

    let launch_info = load_any(
        &mut frame_allocator,
        &mut page_table,
        &kernel_data,
        selected_entry.cmdline.as_deref(),
        alloc::vec![],
    )
    .expect("Falha ao preparar Kernel (Protocol Error)");

    ignite::println!("Saindo dos servicos de boot UEFI...");

    // 11. Exit Boot Services
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

    // 12. Salto para o Kernel
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

// ============================================================================
// Helpers Internos
// ============================================================================

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

    // Retorna o mapa de memória e um iterador vazio
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
