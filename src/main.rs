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
    ignite::println!("[OK] Heap inicializada.");

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

    // 8. Carregar Kernel (Alocação UEFI Direta - Padrão Industrial)
    // ----------------------------------------------------------------
    // Ao invés de usar Vec<u8> no heap do bootloader (limitado a 4MB),
    // alocamos diretamente via UEFI allocate_pool. Isso permite carregar
    // kernels de qualquer tamanho sem desperdício de RAM.

    let mut root_dir = boot_fs.root().expect("Falha raiz FS");
    let mut kernel_file = root_dir
        .open_file(&selected_entry.path)
        .expect("Kernel nao encontrado no disco");

    // 8.1: Obter tamanho exato do kernel via metadata
    let kernel_metadata = kernel_file
        .metadata()
        .expect("Falha ao obter metadata do kernel");
    let kernel_size = kernel_metadata.size as usize;

    ignite::println!(
        "Tamanho do kernel: {} bytes ({} MB)",
        kernel_size,
        kernel_size / (1024 * 1024)
    );

    // 8.2: Validar tamanho (proteção contra kernels malformados ou muito grandes)
    if kernel_size == 0 {
        panic!("Kernel tem tamanho zero! Arquivo corrompido?");
    }
    if kernel_size > ignite::core::config::limits::MAX_KERNEL_SIZE {
        panic!(
            "Kernel muito grande: {} bytes (max: {} bytes)",
            kernel_size,
            ignite::core::config::limits::MAX_KERNEL_SIZE
        );
    }

    // 8.3: Alocar memória UEFI diretamente (LoaderData - será passada ao kernel via
    // memory map)
    let kernel_buffer_ptr = bs
        .allocate_pool(uefi::table::boot::MemoryType::LoaderData, kernel_size)
        .expect("FALHA CRITICA: Nao foi possivel alocar memoria UEFI para o kernel");

    ignite::println!(
        "[OK] Buffer UEFI alocado em: 0x{:X}",
        kernel_buffer_ptr as u64
    );

    // 8.4: Criar slice Rust do buffer UEFI (unsafe: confiamos que UEFI alocou
    // corretamente)
    let kernel_data: &mut [u8] =
        unsafe { core::slice::from_raw_parts_mut(kernel_buffer_ptr as *mut u8, kernel_size) };

    // 8.5: Ler kernel diretamente para o buffer (sem alocações intermediárias)
    ignite::fs::read_exact(kernel_file.as_mut(), kernel_data)
        .expect("Erro de I/O ao ler Kernel para buffer UEFI");

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
            launch_info.use_fixed_redstone_entry,
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

/// Jump para o kernel: escolhe entre Redstone (fixo) ou genérico (dinâmico).
#[no_mangle]
unsafe extern "C" fn jump_to_kernel(
    entry: u64,
    use_fixed: bool,
    stack: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    cr3: u64,
) -> ! {
    if use_fixed {
        // Protocolo Redstone: jump fixo para 0xffffffff80000000
        ignite::println!("[DEBUG] Saltando para o kernel via jump_to_kernel_redstone");
        jump_to_kernel_redstone(stack, arg1, arg2, arg3, arg4, cr3)
    } else {
        // Outros protocolos: jump dinâmico
        ignite::println!(
            "[DEBUG] Usando jump_to_kernel_generic (entry=0x{:X})",
            entry
        );
        jump_to_kernel_generic(entry, stack, arg1, arg2, arg3, arg4, cr3)
    }
}

/// Jump FIXO para Kernel Redstone (0xffffffff80000000).
/// Usado exclusivamente para protocol: redstone no ignite.cfg.
///
/// O kernel Forge sempre está neste endereço por convenção do linker script.
#[no_mangle]
unsafe extern "C" fn jump_to_kernel_redstone(
    stack: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    cr3: u64,
) -> ! {
    // Endereço fixo do kernel Forge (convenção Redstone OS)
    const REDSTONE_KERNEL_ENTRY: u64 = 0xffffffff80000000;

    core::arch::asm!(
        "cli",

        // Carregar CR3 (page table)
        "mov rax, {cr3}",
        "mov cr3, rax",

        // Configurar stack se fornecida
        "test {stack}, {stack}",
        "je 2f",
        "mov rsp, {stack}",
        "xor rbp, rbp",
        "2:",

        // System V AMD64 ABI
        "mov rdi, {arg1}",
        "mov rsi, {arg2}",
        "mov rdx, {arg3}",
        "mov rbx, {arg4}",

        // Jump FIXO para o kernel Redstone
        "mov rax, {redstone_entry}",
        "jmp rax",

        redstone_entry = const REDSTONE_KERNEL_ENTRY,
        stack = in(reg) stack,
        arg1 = in(reg) arg1,
        arg2 = in(reg) arg2,
        arg3 = in(reg) arg3,
        arg4 = in(reg) arg4,
        cr3 = in(reg) cr3,

        options(noreturn)
    );
}

/// Jump GENÉRICO para kernels (Linux, Multiboot2, etc).
/// Usa o entry_point fornecido dinamicamente pelo protocolo.
#[no_mangle]
unsafe extern "C" fn jump_to_kernel_generic(
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

        // Carregar CR3
        "mov rax, {cr3}",
        "mov cr3, rax",

        // Configurar stack
        "test {stack}, {stack}",
        "je 2f",
        "mov rsp, {stack}",
        "xor rbp, rbp",
        "2:",

        // System V AMD64 ABI
        "mov rdi, {arg1}",
        "mov rsi, {arg2}",
        "mov rdx, {arg3}",
        "mov rbx, {arg4}",

        // Jump dinâmico baseado em entry_point
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
