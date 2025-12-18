//! Bootloader Ignite - Biblioteca Principal
//!
//! Bootloader UEFI moderno para Redstone OS, escrito em Rust.
//!
//! # Arquitetura
//!
//! O bootloader é organizado em módulos especializados:
//! - `error`: Sistema de erros centralizado
//! - `types`: Tipos compartilhados
//! - `memory`: Gerenciamento de memória
//! - `video`: Configuração de vídeo (GOP)
//! - `fs`: Sistema de arquivos
//! - `elf`: Parsing e carregamento de ELF
//!
//! # Fluxo de Boot
//!
//! 1. Inicialização UEFI
//! 2. Carregamento do kernel (ELF)
//! 3. Configuração de vídeo
//! 4. Carregamento do InitFS (opcional)
//! 5. Preparação de argumentos
//! 6. Transferência de controle para o kernel

#![no_std]
#![no_main]

extern crate alloc;

pub mod boot_info;
// pub mod config; // TODO: Debug
pub mod elf;
pub mod error;
pub mod fs;
pub mod memory;
pub mod recovery;
// pub mod security; // TODO: Debug
pub mod types;
// pub mod ui; // TODO: Debug
pub mod video;

// use crate::config::BootConfig; // TODO: Debug
use log::info;
use uefi::{prelude::*, table::boot::MemoryType};

// use crate::security::{IntegrityChecker, RollbackProtection, SecureBootManager}; // TODO:
// Debug
use crate::types::KernelArgs;
// use crate::ui::BootMenu; // TODO: Debug
use crate::video::{GopVideoOutput, VideoOutput};
use crate::{
    elf::ElfLoader,
    error::Result,
    fs::{FileLoader, InitFsLoader},
    memory::MemoryAllocator,
    recovery::{BootOptions, Diagnostics, KeyDetector},
};

/// Função principal do bootloader
///
/// Orquestra todo o processo de boot: carregamento de arquivos, parsing ELF,
/// configuração de vídeo e transferência de controle para o kernel.
///
/// # Argumentos
/// * `image_handle` - Handle da imagem do bootloader
/// * `mut system_table` - Tabela de sistema UEFI
///
/// # Retorna
/// Result indicando sucesso ou tipo de erro
pub fn boot(image_handle: Handle, mut system_table: SystemTable<Boot>) -> ! {
    // Inicializar serviços UEFI
    uefi::helpers::init(&mut system_table).unwrap();
    system_table.stdout().reset(false).unwrap();

    info!("═══════════════════════════════════════════════════");
    info!("  Bootloader Ignite v0.2.0 - Redstone OS");
    info!("═══════════════════════════════════════════════════");

    // Mostrar hint de tecla de recovery (estilo Ctrl+Alt+Del)
    KeyDetector::show_recovery_hint();

    // Mostrar hint de tecla de configuração (canto inferior esquerdo)
    // TODO: Descomentar após implementar BootMenu
    // BootMenu::show_config_hint();

    // TODO: Verificar se tecla 'R' foi pressionada para entrar em modo recovery
    // if KeyDetector::check_recovery_key(boot_services) {
    //     enter_recovery_mode(...);
    // }

    // TODO: Verificar se tecla 'C' foi pressionada para configuração
    // if KeyDetector::check_config_key(boot_services) {
    //     enter_config_mode(...);
    // }

    let boot_services = system_table.boot_services();

    // 1. Criar alocador de memória
    let allocator = MemoryAllocator::new(boot_services);

    // 1.1 Carregar configuração de boot
    // TODO: Carregar de arquivo boot.cfg ou ignite.ini
    // TODO: Descomentar após implementar BootConfig
    // let config = BootConfig::default();
    // info!("Menu de boot: {}", if config.menu.enabled { "habilitado" } else {
    // "desabilitado" });

    // 1.2 Detectar sistemas operacionais disponíveis
    // TODO: Implementar detecção automática de Linux/Windows
    // let os_list = OsList::detect(&mut file_loader);
    // info!("Sistemas detectados: {}", os_list.count);

    // 1.3 Verificar se deve mostrar menu
    // TODO: Verificar se tecla foi pressionada
    // let show_menu = config.should_show_menu(false); // false = nenhuma tecla
    // pressionada

    // if show_menu {
    //     info!("Exibindo menu de boot...");
    //     // TODO: Implementar menu interativo
    //     // let selected_os = BootMenu::show(&config, &os_list);
    //     // Carregar OS selecionado
    // }

    // 1.4 Inicializar opções de boot com fallback
    // TODO: Carregar contador de tentativas de variável UEFI
    let mut boot_options = BootOptions::default();
    info!(
        "Sistema de fallback: {} tentativas máximas",
        boot_options.max_attempts
    );

    // 2. Executar diagnóstico básico (opcional, não bloqueia)
    info!("Etapa 1/6: Diagnóstico do sistema...");
    let mut file_loader = FileLoader::new(boot_services, image_handle, &allocator)
        .expect("Falha ao criar file loader");
    Diagnostics::run_basic_diagnostics(&mut file_loader);

    // 3. Carregar kernel (usando fallback se necessário)
    info!("Etapa 2/6: Carregando kernel...");
    let kernel_entry = boot_options.select_kernel();
    info!("Kernel selecionado: {}", kernel_entry.name);
    let kernel_file = file_loader
        .load_file(kernel_entry.path)
        .expect("Falha ao carregar kernel");

    // 4. Parsear e carregar ELF
    info!("Etapa 3/6: Parseando e carregando ELF...");
    let kernel_data =
        unsafe { core::slice::from_raw_parts(kernel_file.ptr as *const u8, kernel_file.size) };

    // TODO: Reativar verificações de segurança após debug
    // 4.1 Verificações de segurança (modo permissivo)
    // info!("Executando verificações de segurança...");
    //
    // Verificar integridade (SHA-256)
    // let _integrity_result = IntegrityChecker::verify_file(kernel_data, None);
    // TODO: Carregar hash esperado de manifesto
    //
    // Verificar versão (proteção contra rollback)
    // if let Some(version) = RollbackProtection::extract_version(kernel_data) {
    // info!("Versão do kernel: {}.{}.{}", version.major, version.minor,
    // version.patch); let _rollback_result =
    // RollbackProtection::check_version(version); TODO: Salvar versão em
    // variável UEFI após boot bem-sucedido }
    //
    // Verificar estado do Secure Boot
    // let sb_state = SecureBootManager::get_state();
    // info!("Secure Boot: {}", sb_state.as_str());
    // TODO: Se Secure Boot habilitado, validar assinatura

    // Continuar com carregamento ELF
    let elf_loader = ElfLoader::new(&allocator);
    let loaded_kernel = elf_loader.load(kernel_data).expect("Falha ao carregar ELF");

    // 5. Configurar vídeo
    info!("Etapa 4/6: Configurando vídeo...");
    let mut video = GopVideoOutput::new(boot_services, image_handle);
    video.initialize().expect("Falha ao inicializar vídeo");
    let framebuffer = video.get_framebuffer();
    info!(
        "Framebuffer: {}x{} em {:#x}",
        framebuffer.horizontal_resolution, framebuffer.vertical_resolution, framebuffer.ptr
    );

    // 6. Carregar InitFS (opcional)
    info!("Etapa 5/6: Carregando InitFS...");
    let initfs = InitFsLoader::load(&mut file_loader).expect("Falha ao carregar InitFS");

    // 7. Preparar argumentos do kernel
    info!("Etapa 6/6: Preparando argumentos do kernel...");
    let args = prepare_kernel_args(&allocator, &loaded_kernel, &initfs)
        .expect("Falha ao preparar argumentos");

    // 7.5 Criar BootInfo com memory map REAL da UEFI
    const MEMORY_MAP_ADDR: usize = 0x9000;
    const MAX_REGIONS: usize = 256;

    // Obter memory map da UEFI
    let map_size = boot_services.memory_map_size();
    let mut map_buffer = alloc::vec![0u8; map_size.map_size + 10 * map_size.entry_size];
    let memory_map = boot_services
        .memory_map(&mut map_buffer)
        .expect("Failed to get UEFI memory map");

    // Converter para nosso formato
    let memory_regions = unsafe {
        core::slice::from_raw_parts_mut(
            MEMORY_MAP_ADDR as *mut boot_info::MemoryRegion,
            MAX_REGIONS,
        )
    };

    let mut region_count = 0;
    for desc in memory_map.entries() {
        if region_count >= MAX_REGIONS {
            break;
        }

        let region_type = match desc.ty {
            MemoryType::CONVENTIONAL => boot_info::MemoryRegionType::Usable,
            MemoryType::ACPI_RECLAIM => boot_info::MemoryRegionType::AcpiReclaimable,
            MemoryType::ACPI_NON_VOLATILE => boot_info::MemoryRegionType::AcpiNvs,
            _ => boot_info::MemoryRegionType::Reserved,
        };

        memory_regions[region_count] =
            boot_info::MemoryRegion::new(desc.phys_start, desc.page_count * 4096, region_type);
        region_count += 1;
    }

    info!("Memory map: {} regions collected from UEFI", region_count);

    // Criar e escrever BootInfo
    let boot_info = boot_info::BootInfo {
        fb_addr:         framebuffer.ptr,
        fb_width:        framebuffer.horizontal_resolution as u32,
        fb_height:       framebuffer.vertical_resolution as u32,
        fb_stride:       framebuffer.stride as u32,
        fb_format:       0,
        kernel_base:     loaded_kernel.base_address,
        kernel_size:     loaded_kernel.size,
        initfs_addr:     initfs.as_ref().map(|f| f.ptr).unwrap_or(0),
        initfs_size:     initfs.as_ref().map(|f| f.size as u64).unwrap_or(0),
        memory_map_addr: MEMORY_MAP_ADDR as u64,
        memory_map_size: region_count as u64,
    };

    unsafe {
        boot_info.write();
    }
    info!("BootInfo written to 0x8000");

    // 8. Desativar watchdog timer
    boot_services
        .set_watchdog_timer(0, 0x10000, None)
        .unwrap_or(());

    // 9. Logging final
    info!("═══════════════════════════════════════════════════");
    info!("  Boot completo. Transferindo controle...");
    info!("  Entry Point: {:#x}", loaded_kernel.entry_point);
    unsafe {
        info!("  Kernel Base: {:#x}", (*args).kernel_base);
        info!("  Kernel Size: {:#x}", (*args).kernel_size);
        info!(
            "  InitFS: {:#x} ({} bytes)",
            (*args).bootstrap_base,
            (*args).bootstrap_size
        );
    }
    info!("═══════════════════════════════════════════════════");

    // 10. Sair dos serviços de boot
    // TODO: Se boot for bem-sucedido (kernel assume controle),
    // resetar contador de tentativas em próximo boot
    let (_rt, _map) = system_table.exit_boot_services(MemoryType::LOADER_DATA);

    // 11. Saltar para o kernel usando função naked
    // IMPORTANTE: Inline assembly não funciona, usar naked function
    unsafe {
        jump_to_kernel_naked(loaded_kernel.entry_point, 0x8000);
    }
}

/// Função naked para saltar para o kernel
///
/// IMPORTANTE: Esta função usa naked para garantir que o assembly
/// seja exatamente como escrevemos, sem prólogo/epílogo do compilador
#[unsafe(naked)]
extern "C" fn jump_to_kernel_naked(entry: u64, boot_info: u64) -> ! {
    unsafe {
        core::arch::naked_asm!(
            // entry está em RDI (primeiro argumento)
            // boot_info está em RSI (segundo argumento)
            "mov rax, rdi", // RAX = entry point
            "mov rdi, rsi", // RDI = boot_info (argumento para kernel)
            "xor rsi, rsi", // RSI = 0
            "jmp rax",      // Saltar para entry point!
        );
    }
}

/// Prepara a estrutura KernelArgs
fn prepare_kernel_args(
    allocator: &MemoryAllocator,
    loaded_kernel: &types::LoadedKernel,
    initfs: &Option<types::LoadedFile>,
) -> Result<*const KernelArgs> {
    // Alocar memória para KernelArgs
    let args_ptr = allocator.allocate_any(1)?;
    let args = unsafe { &mut *(args_ptr as *mut KernelArgs) };

    // Preencher estrutura
    args.kernel_base = loaded_kernel.base_address;
    args.kernel_size = loaded_kernel.size;
    args.stack_base = 0; // Stack será configurada pelo kernel
    args.stack_size = 0;
    args.env_base = 0;
    args.env_size = 0;
    args.hwdesc_base = 0; // TODO: Encontrar RSDP
    args.hwdesc_size = 0;
    args.areas_base = 0;
    args.areas_size = 0;

    // Configurar InitFS se disponível
    if let Some(initfs_file) = initfs {
        args.bootstrap_base = initfs_file.ptr;
        args.bootstrap_size = initfs_file.size as u64;
    } else {
        args.bootstrap_base = 0;
        args.bootstrap_size = 0;
    }

    Ok(args as *const KernelArgs)
}
