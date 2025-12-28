//! # Redstone OS Bootloader (Ignite) - Entry Point
//!
//! O `Ignite` é o primeiro estágio de software controlado por nós.
//! Ele não é apenas um carregador; é o **Guardião da Integridade** do sistema.
//!
//! ## 🎯 Missão Crítica
//! 1. **Sanitização:** Limpar o estado "sujo" deixado pelo firmware UEFI.
//! 2. **Verificação:** Garantir que o Kernel é autêntico (Secure Boot /
//!    Hashing).
//! 3. **Mapeamento:** Preparar o mapa de memória físico e virtual para o
//!    Kernel.
//! 4. **Handoff:** Passar o controle de forma irreversível
//!    (`ExitBootServices`).
//!
//! ## 🏗️ Fluxo de Execução (The Boot Pipeline)
//!
//! 1. **Early init:** Inicializa Serial (COM1) para logs de debug. O usuário
//!    não vê, mas nós vemos.
//! 2. **Heap Setup:** Aloca um pool inicial (Bump Allocator) para structs do
//!    Rust (`Box`, `Vec`).
//! 3. **Config Loading:** Busca `ignite.cfg` no ESP. Se falhar, entra
//!    automático em **Recovery Mode**.
//! 4. **Video Handshake:** Negocia o modo de vídeo GOP (Graphics Output
//!    Protocol). O Kernel *não* toca na BIOS/UEFI de vídeo.
//! 5. **UI/Menu:** Renderiza o menu de seleção (se não for `quiet`).
//! 6. **Kernel Loading:**
//!     - Lê o kernel para um buffer UEFI (`LoaderData`).
//!     - **CRÍTICO:** Valida assinatura criptográfica (se Secure Policy estiver
//!       ativa).
//! 7. **Memory Map Capture:** Obtém o mapa de memória final da UEFI.
//! 8. **Point of No Return:** Chama `ExitBootServices()`. A partir daqui, o
//!    firmware UEFI morre.
//! 9. **Trampoline:** Salto para `0xffffffff80000000` (Redstone) ou outro entry
//!    point (Chainload).
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Robustez na Alocação:** O kernel é carregado diretamente em páginas
//!   alocadas via UEFI (`allocate_pool`), evitando cópias duplas e fragmentação
//!   do heap do bootloader.
//! - **Fail-Safe:** O sistema de configuração tem fallback automático para
//!   `Recovery` se o parser falhar.
//! - **Observabilidade:** Logs são enviados para Serial *e* Vídeo desde o
//!   primeiro milissegundo.
//!
//! ### ⚠️ Pontos de Atenção (Riscos e Dívida Técnica)
//! - **Race Condition no ExitBootServices:** Existe uma janela minúscula onde,
//!   se uma interrupção ocorrer entre `get_memory_map` e `exit_boot_services`,
//!   a chamada falha. *Status:* O código tenta corrigir com um retry loop, mas
//!   o ideal seria desabilitar interrupções antes.
//! - **Argument Chaos:** `jump_to_kernel` passa 6 argumentos via registradores.
//!   Isso é frágil. *Melhoria:* Passar um único ponteiro para `BootInfo` no
//!   registro `RDI` (Convenção System V).
//! - **Stack Size:** O stack do bootloader é definido pelo firmware. Se
//!   recursarmos muito ou alocarmos arrays grandes na stack, causaremos **Stack
//!   Overflow** silencioso.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Reliability)** Implementar **Watchdog Timer** durante o boot.
//!   - *Motivo:* Se o kernel travar no early init, o PC não deve congelar; deve
//!     resetar após 10s.
//! - [ ] **TODO: (Architecture)** Migrar `jump_to_kernel` para usar apenas
//!   `BootInfo`.
//!   - *Impacto:* Simplifica a ABI e permite passar mais dados (ACPI,
//!     Framebuffer) sem usar todos os registros da CPU.
//! - [ ] **TODO: (Security)** Implementar **TPM Measurement**.
//!   - *Meta:* Estender o PCR do TPM com o hash do Kernel carregado antes de
//!     executá-lo.

#![no_std]
#![no_main]
#![feature(alloc_error_handler)] // Habilita o handler de OOM customizado

extern crate alloc;

// Imports da biblioteca Ignite
use ignite::{
    config::{loader::load_configuration, BootConfig, Protocol},
    core::{
        handoff::FramebufferInfo as HandoffFbInfo, // Alias para evitar colisão
        logging,
    },
    fs::{FileSystem, UefiFileSystem},
    memory::{BumpAllocator, PageTableManager, UefiFrameAllocator},
    protos::load_any,
    recovery::Diagnostics,
    security::{validate_and_measure, SecurityPolicy},
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
            .expect("[FAIL] Nao foi possivel alocar Heap inicial");

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
        .expect("[FAIL] Falha ao abrir LoadedImage");

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
        .expect("[FAIL] Falha ao abrir SimpleFileSystem");

    let fs_proto_ref =
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
    let (_gop, fb_info) = video::init_video(bs).expect("[FAIL] Nao foi possivel iniciar Video GOP");

    // Preparar estrutura de Handoff para o Kernel (e UI)
    let handoff_fb_info = HandoffFbInfo {
        addr:   fb_info.addr,
        size:   fb_info.size as u64,
        width:  fb_info.width,
        height: fb_info.height,
        stride: fb_info.stride,
        format: match fb_info.format {
            ignite::video::PixelFormat::RgbReserved8Bit => ignite::core::handoff::PixelFormat::Rgb,
            ignite::video::PixelFormat::BgrReserved8Bit => ignite::core::handoff::PixelFormat::Bgr,
            ignite::video::PixelFormat::Bitmask => ignite::core::handoff::PixelFormat::Bitmask,
            ignite::video::PixelFormat::BltOnly => ignite::core::handoff::PixelFormat::BltOnly,
        },
    };

    // 6. Interface de Usuário (Menu Gráfico)
    let selected_entry = if !config.quiet && config.timeout.unwrap_or(0) > 0 {
        let fb_ptr = fb_info.addr;
        let mut menu = Menu::new(&config);
        // Reuse handoff_fb_info (Copy trait required or clone)
        // HandoffFbInfo derives Copy/Clone
        unsafe { menu.run(fb_ptr, handoff_fb_info) }
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

    let mut root_dir = boot_fs.root().expect("[FAIL] Falha raiz FS");
    let mut kernel_file = root_dir
        .open_file(&selected_entry.path)
        .expect("[FAIL] Kernel nao encontrado no disco");

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
        panic!("[FAIL] Kernel tem tamanho zero! Arquivo corrompido?");
    }
    if kernel_size > ignite::core::config::limits::MAX_KERNEL_SIZE {
        panic!(
            "[FAIL] Kernel muito grande: {} bytes (max: {} bytes)",
            kernel_size,
            ignite::core::config::limits::MAX_KERNEL_SIZE
        );
    }

    // 8.3: Alocar memória UEFI diretamente (LoaderData - será passada ao kernel via
    // memory map)
    let kernel_buffer_ptr = bs
        .allocate_pool(uefi::table::boot::MemoryType::LoaderData, kernel_size)
        .expect("[FAIL] Nao foi possivel alocar memoria UEFI para o kernel");

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
        .expect("[FAIL] Erro de I/O ao ler Kernel para buffer UEFI");

    // 8.6: Carregar Módulos (InitRD, Drivers)
    let mut loaded_modules = alloc::vec::Vec::new();
    for module_cfg in &selected_entry.modules {
        ignite::println!("Carregando modulo: {}", module_cfg.path);

        let mut module_file = root_dir
            .open_file(&module_cfg.path)
            .expect("[FAIL] Modulo nao encontrado no disco");

        let mod_meta = module_file
            .metadata()
            .expect("[FAIL] Falha ao obter metadata do modulo");
        let mod_size = mod_meta.size as usize;

        ignite::println!("Tamanho: {} bytes ({} KB)", mod_size, mod_size / 1024);

        if mod_size == 0 {
            ignite::println!("AVISO: Modulo vazio ignorado.");
            continue;
        }

        let mod_buffer_ptr = bs
            .allocate_pool(uefi::table::boot::MemoryType::LoaderData, mod_size)
            .expect("[FAIL] OOM ao alocar memoria para modulo");

        let mod_data: &mut [u8] =
            unsafe { core::slice::from_raw_parts_mut(mod_buffer_ptr as *mut u8, mod_size) };

        ignite::fs::read_exact(module_file.as_mut(), mod_data)
            .expect("[FAIL] Erro de I/O ao ler modulo");

        loaded_modules.push(ignite::core::types::LoadedFile {
            ptr:  mod_buffer_ptr as u64,
            size: mod_size,
        });

        ignite::println!("[OK] Modulo carregado em: 0x{:X}", mod_buffer_ptr as u64);
    }

    // 9. Segurança
    let policy = SecurityPolicy::new(&config);
    if let Err(e) = validate_and_measure(&kernel_data, &selected_entry.name, &policy) {
        panic!("[FAIL] Violacao de Seguranca detectada: {:?}", e);
    }
    // TODO: Validar módulos também

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
            panic!("[FAIL] Falha ao carregar imagem EFI: {:?}", status);
        }

        // Iniciar a imagem
        let mut exit_data_size: usize = 0;
        let mut exit_data: *mut u16 = core::ptr::null_mut();

        // Passa o controle para o aplicativo EFI (Shell)
        let status =
            unsafe { (bs.start_image_f)(child_handle, &mut exit_data_size, &mut exit_data) };

        if status.is_error() {
            ignite::println!("[FAIL] Aplicacao EFI retornou erro: {:?}", status);
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

    // CRÍTICO: Capturar Memory Map ANTES de exit_boot_services
    // O kernel precisa saber quais regiões de memória estão disponíveis
    let memory_map_buffer = capture_memory_map(bs);

    let launch_info = load_any(
        &mut frame_allocator,
        &mut page_table,
        &kernel_data,
        selected_entry.cmdline.as_deref(),
        loaded_modules,
        memory_map_buffer,     // Passa o memory map
        Some(handoff_fb_info), // Passa Framebuffer Info
    )
    .expect("[FAIL] Falha ao preparar Kernel (Protocol Error)");

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

    let _ = unsafe {
        (bs.get_memory_map_f)(
            &mut map_size,
            core::ptr::null_mut(),
            &mut map_key,
            &mut descriptor_size,
            &mut descriptor_version,
        )
    };

    // Retorna o mapa de memória e um iterador vazio
    (map_key, core::iter::empty())
}

/// Captura o Memory Map do UEFI em um buffer persistente.
/// Retorna (ponteiro, contagem de entradas).
fn capture_memory_map(bs: &ignite::uefi::BootServices) -> (u64, u64) {
    use ignite::core::handoff::MemoryMapEntry;

    let mut map_size = 0;
    let mut map_key = 0;
    let mut descriptor_size = 0;
    let mut descriptor_version = 0;

    // 1. Descobrir tamanho necessário
    let _ = unsafe {
        (bs.get_memory_map_f)(
            &mut map_size,
            core::ptr::null_mut(),
            &mut map_key,
            &mut descriptor_size,
            &mut descriptor_version,
        )
    };

    // 2. Alocar buffer (com margem de segurança)
    map_size += descriptor_size * 10;
    let buffer_ptr = bs
        .allocate_pool(ignite::uefi::table::boot::MemoryType::LoaderData, map_size)
        .expect("[FAIL] Falha ao alocar buffer para memory map");

    // 3. Obter memory map real
    let status = unsafe {
        (bs.get_memory_map_f)(
            &mut map_size,
            buffer_ptr as *mut ignite::uefi::table::boot::MemoryDescriptor,
            &mut map_key,
            &mut descriptor_size,
            &mut descriptor_version,
        )
    };

    if status.is_error() {
        ignite::println!("[FAIL] Falha ao capturar memory map!");
        return (0, 0);
    }

    // 4. Converter entradas UEFI para formato do Forge
    let num_descriptors = map_size / descriptor_size;

    // Alocar array de MemoryMapEntry
    let entries_size = num_descriptors * core::mem::size_of::<MemoryMapEntry>();
    let entries_ptr =
        bs.allocate_pool(
            ignite::uefi::table::boot::MemoryType::LoaderData,
            entries_size,
        )
        .expect("[FAIL] Falha ao alocar array de memory map") as *mut MemoryMapEntry;

    let forge_entries = unsafe { core::slice::from_raw_parts_mut(entries_ptr, num_descriptors) };

    // ============================================================
    // DEBUG: Ative (true) ou desative (false) o log detalhado
    // ============================================================
    const DEBUG_MEMORY_MAP: bool = false;

    // 5. Converter cada entrada - IMPORTANTE: usar descriptor_size, não sizeof!
    let mut valid_entries = 0;
    let mut total_usable_ram: u64 = 0;
    let mut max_ram_address: u64 = 0;

    if DEBUG_MEMORY_MAP {
        ignite::println!("=== DEBUG: Analisando Memory Map UEFI ===");
        ignite::println!("Descriptor size: {} bytes", descriptor_size);
    }

    // Iterar manualmente usando descriptor_size (pode ser maior que
    // sizeof(MemoryDescriptor))
    for i in 0..num_descriptors {
        use ignite::uefi::table::boot::MemoryType;

        // Calcular ponteiro para esta entrada usando descriptor_size
        let desc_ptr = unsafe {
            (buffer_ptr as *const u8).add(i * descriptor_size)
                as *const ignite::uefi::table::boot::MemoryDescriptor
        };
        let desc = unsafe { &*desc_ptr };

        // Validação: Ignorar entradas claramente corrompidas
        const MAX_REASONABLE_ADDR: u64 = 1024 * 1024 * 1024 * 1024; // 1 TB
        const MAX_REGION_SIZE: u64 = 128 * 1024 * 1024 * 1024; // 128 GB por região

        if desc.physical_start > MAX_REASONABLE_ADDR {
            if DEBUG_MEMORY_MAP {
                ignite::println!(
                    "  [{:3}] IGNORADO: Base address absurdo ({:#x})",
                    i,
                    desc.physical_start
                );
            }
            continue;
        }

        if desc.number_of_pages == 0 {
            if DEBUG_MEMORY_MAP {
                ignite::println!("  [{:3}] IGNORADO: Região vazia", i);
            }
            continue;
        }

        // Validar que o tamanho também é razoável
        let size = desc.number_of_pages * 4096;
        if size > MAX_REGION_SIZE {
            if DEBUG_MEMORY_MAP {
                ignite::println!(
                    "  [{:3}] IGNORADO: Tamanho absurdo ({} MB)",
                    i,
                    size / (1024 * 1024)
                );
            }
            continue;
        }

        // Debug detalhado (se ativado)
        if DEBUG_MEMORY_MAP {
            let type_name = match desc.ty {
                ty if ty == MemoryType::ConventionalMemory as u32 => "ConventionalMemory",
                ty if ty == MemoryType::LoaderData as u32 => "LoaderData",
                ty if ty == MemoryType::LoaderCode as u32 => "LoaderCode",
                ty if ty == MemoryType::BootServicesData as u32 => "BootServicesData",
                ty if ty == MemoryType::BootServicesCode as u32 => "BootServicesCode",
                ty if ty == MemoryType::ACPIReclaimMemory as u32 => "ACPIReclaim",
                ty if ty == MemoryType::ACPIMemoryNVS as u32 => "ACPINVS",
                _ => "Other",
            };

            ignite::println!(
                "  [{:3}] {:20} Base:{:#016x} Pages:{:#010x} Size:{} MB",
                i,
                type_name,
                desc.physical_start,
                desc.number_of_pages,
                size / (1024 * 1024)
            );
        }

        // Contabilizar RAM usável E calcular endereço máximo APENAS com RAM real
        if desc.ty == MemoryType::ConventionalMemory as u32 {
            total_usable_ram += size;

            // Calcular endereço máximo APENAS da RAM utilizável
            let end = desc.physical_start + size;
            if end > max_ram_address {
                max_ram_address = end;
            }
        }

        forge_entries[valid_entries] = MemoryMapEntry {
            base: desc.physical_start,
            len:  size,
            typ:  match desc.ty {
                ty if ty == MemoryType::ConventionalMemory as u32 => {
                    ignite::core::handoff::MemoryType::Usable
                },
                ty if ty == MemoryType::LoaderData as u32
                    || ty == MemoryType::LoaderCode as u32 =>
                {
                    ignite::core::handoff::MemoryType::BootloaderReclaimable
                },
                ty if ty == MemoryType::ACPIReclaimMemory as u32 => {
                    ignite::core::handoff::MemoryType::AcpiReclaimable
                },
                ty if ty == MemoryType::ACPIMemoryNVS as u32 => {
                    ignite::core::handoff::MemoryType::AcpiNvs
                },
                _ => ignite::core::handoff::MemoryType::Reserved,
            },
        };

        valid_entries += 1;
    }

    // Sempre mostrar resumo
    ignite::println!("Memory map: {} entradas válidas", valid_entries);
    ignite::println!(
        "RAM utilizável: {} MB ({} GB)",
        total_usable_ram / (1024 * 1024),
        total_usable_ram / (1024 * 1024 * 1024)
    );

    (entries_ptr as u64, valid_entries as u64)
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
        ignite::println!("[JUMP] Saltando para o kernel via jump_to_kernel_redstone");
        jump_to_kernel_redstone(stack, arg1, arg2, arg3, arg4, cr3)
    } else {
        // Outros protocolos: jump dinâmico
        ignite::println!("[JUMP] Usando jump_to_kernel_generic (entry=0x{:X})", entry);
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
