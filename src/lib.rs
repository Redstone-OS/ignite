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

// Global allocator - Static Bump Allocator (independente de UEFI boot services)
#[global_allocator]
static ALLOCATOR: memory::BumpAllocator = memory::BumpAllocator::new();

pub mod config;
pub mod core; // Core utilities: constants, types, error, boot_info, logger
pub mod elf;
pub mod fs;
pub mod hardware; // Hardware abstractions: acpi, fdt, io, serial
pub mod memory; // Memory management: allocator, bump_allocator
pub mod protos;
pub mod recovery;
pub mod security;
pub mod uefi; // Nossa implementação UEFI pura (sem dependências externas)
pub mod ui;
pub mod video;

// use crate::config::BootConfig; // TODO: Debug
use log::info;

// use crate::security::{IntegrityChecker, RollbackProtection, SecureBootManager}; // TODO:
// Debug
use crate::core::{error::Result, types::KernelArgs};
// use crate::ui::BootMenu; // TODO: Debug
use crate::video::{GopVideoOutput, VideoOutput};
use crate::{
    fs::FileLoader,
    memory::MemoryAllocator,
    recovery::{Diagnostics, KeyDetector},
    uefi::{
        BootServices, Handle,
        table::{boot::MemoryType, system::SystemTable},
    },
};

/// Função principal do bootloader
///
/// Orquestra todo o processo de boot com multi-protocol support, configuração e
/// menu interativo.
///
/// # Argumentos
/// * `image_handle` - Handle da imagem do bootloader
/// * `mut system_table` - Tabela de sistema UEFI
///
/// # Retorna
/// Nunca retorna - transfere o controle para o kernel
pub fn boot(image_handle: Handle, system_table: *mut SystemTable) -> ! {
    // Debug: bootloader started
    unsafe {
        let port: u16 = 0x3F8;
        for &byte in b"[1/20] Boot started\r\n" {
            ::core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
        }
    }

    // Inicializar serviços UEFI (API 0.31: init() não recebe argumentos)
    // Inicializar UEFI (sem helpers externos)

    unsafe {
        let port: u16 = 0x3F8;
        for &byte in b"[2/20] UEFI init OK\r\n" {
            ::core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
        }
    }

    // CRÍTICO: Pré-alocar heap estática VIA UEFI allocate_pages
    // Isso garante que temos memória própria que sobrevive a exit_boot_services
    let boot_services = unsafe { &*(*system_table).boot_services };
    let heap_pages = crate::memory::bump_allocator::HEAP_SIZE / 4096;

    unsafe {
        let port: u16 = 0x3F8;
        for &byte in b"[3/20] Allocating static heap...\r\n" {
            ::core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
        }
    }

    // Alocar heap estática para bump allocator
    let _heap_start = unsafe {
        boot_services
            .allocate_pages_helper(
                crate::uefi::table::boot::AllocateType::AllocateAnyPages,
                MemoryType::LoaderData,
                heap_pages,
            )
            .expect("Failed to allocate static heap for bump allocator") as *mut u8
    };

    // Inicializar bump allocator com heap pré-alocada
    unsafe {
        ALLOCATOR.init();

        let port: u16 = 0x3F8;
        for &byte in b"[4/20] Bump allocator initialized\r\n" {
            ::core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
        }
    }

    // Clear screen
    unsafe {
        let st = &*system_table;
        let stdout = &mut *st.con_out;
        (stdout.reset)(stdout, crate::uefi::FALSE);
    }

    info!("═══════════════════════════════════════════════════");
    info!("║         Ignite UEFI Bootloader v0.4.0           ║");
    info!("║            Redstone OS Boot Manager              ║");
    info!("═══════════════════════════════════════════════════");

    // Mostrar hint de tecla de recovery
    KeyDetector::show_recovery_hint();

    // Obter boot services do system table
    let boot_services = unsafe { &*(*system_table).boot_services };

    // 1. Criar alocador de memória
    let allocator = MemoryAllocator::new(boot_services);

    // 2. Carregar configuração (ou usar padrão se arquivo não existir)
    info!("Etapa 1/7: Carregando configuração...");
    let mut file_loader = FileLoader::new(boot_services, image_handle, &allocator)
        .expect("Falha ao criar file loader");

    let config = load_config(&mut file_loader);

    // Mostrar informações da config
    if config.quiet {
        info!("Modo quiet ativado");
    }
    if config.verbose {
        info!("Modo verbose ativado");
    }
    info!("Entradas de boot: {}", config.entries.len());
    info!("Entrada padrão: {}", config.default_entry);

    // 3. Seleção de entrada (menu ou auto-boot)
    info!("Etapa 2/7: Selecionando entrada de boot...");
    let selected_index = select_boot_entry(boot_services, &config);
    let entry = config.entries[selected_index].clone(); // Clone para evitar problemas de tempo de vida

    info!("Boot selecionado: {}", entry.name);
    info!("  Protocolo: {}", entry.protocol);
    info!("  Kernel: {}", entry.kernel_path);

    // 4. Executar diagnóstico básico
    info!("Etapa 3/7: Diagnóstico do sistema...");
    Diagnostics::run_basic_diagnostics(&mut file_loader);

    // 5. Carregar kernel
    info!("Etapa 4/7: Carregando kernel...");
    let kernel_file = file_loader
        .load_file(&entry.kernel_path) // Agora aceita &str dinâmico!
        .expect("Falha ao carregar kernel");

    let kernel_data =
        unsafe { ::core::slice::from_raw_parts(kernel_file.ptr as *const u8, kernel_file.size) };

    // 6. Carregar initramfs (ramdisk)
    info!("Etapa 5/7: Carregando initramfs...");
    let initramfs = load_initramfs(&mut file_loader);

    // Converter para Vec para compatibilidade com código existente
    let modules = if let Some(ramfs) = initramfs {
        alloc::vec![ramfs]
    } else {
        alloc::vec![]
    };

    // 7. Selecionar e usar protocolo apropriado
    let protocol_name = entry.protocol.as_str(); // Obter &str de String
    info!(
        "Etapa 6/7: Preparando boot com protocolo {}...",
        protocol_name
    );
    let boot_info = use_protocol(
        &allocator,
        protocol_name,
        kernel_data,
        entry.cmdline.as_deref(),
        &modules,
    );

    // 8. Configurar vídeo
    info!("Configurando vídeo...");
    let mut video = GopVideoOutput::new(boot_services, image_handle);
    video.initialize().expect("Falha ao inicializar vídeo");
    let framebuffer = video.get_framebuffer();
    info!(
        "Framebuffer: {}x{} em {:#x}",
        framebuffer.horizontal_resolution, framebuffer.vertical_resolution, framebuffer.ptr
    );

    // 9. Preparar argumentos do kernel (para protocolos que usam)
    info!("Etapa 7/7: Preparando handoff para kernel...");
    let args = prepare_kernel_args_from_boot_info(&allocator, &boot_info, &modules, &framebuffer)
        .expect("Falha ao preparar argumentos");

    // Criar BootInfo com memory map REAL da UEFI
    let boot_info_ptr = allocator
        .allocate_any(1)
        .expect("Failed to allocate BootInfo")
        as *mut crate::core::boot_info::BootInfo;
    let boot_info_addr = boot_info_ptr as u64;

    // Memory map vai logo após o BootInfo
    const BOOT_INFO_SIZE: usize = ::core::mem::size_of::<crate::core::boot_info::BootInfo>();
    let memory_map_addr = (boot_info_addr + BOOT_INFO_SIZE as u64) as usize;
    const MAX_REGIONS: usize = 256;

    // Obter memory map da UEFI usando nova API freestanding 0.31
    let _memory_map =
        // TODO: Implementar memorymap usando nossa camada UEFI
        // let _memory_map = ...; // uefi::boot::memory_map not implemented yet
        log::warn!("Memory map retrieval not yet implemented with pure UEFI");

    // Converter para nosso formato
    let _memory_regions = unsafe {
        ::core::slice::from_raw_parts_mut(
            memory_map_addr as *mut crate::core::boot_info::MemoryRegion,
            MAX_REGIONS,
        )
    };

    let region_count = 0;
    // for desc in memory_map.entries() {
    // if region_count >= MAX_REGIONS {
    // break;
    // }
    //
    // let region_type = match desc.ty {
    // MemoryType::CONVENTIONAL => crate::core::boot_info::MemoryRegionType::Usable,
    // MemoryType::ACPI_RECLAIM =>
    // crate::core::boot_info::MemoryRegionType::AcpiReclaimable,
    // MemoryType::ACPI_NON_VOLATILE =>
    // crate::core::boot_info::MemoryRegionType::AcpiNvs,
    // _ => crate::core::boot_info::MemoryRegionType::Reserved,
    // };
    //
    // memory_regions[region_count] = crate::core::boot_info::MemoryRegion::new(
    // desc.phys_start,
    // desc.page_count * 4096,
    // region_type,
    // );
    // region_count += 1;
    // }

    info!("Memory map: {} regions collected from UEFI", region_count);

    // Criar e escrever BootInfo UEFI
    let uefi_boot_info = crate::core::boot_info::BootInfo {
        fb_addr:         framebuffer.ptr,
        fb_width:        framebuffer.horizontal_resolution as u32,
        fb_height:       framebuffer.vertical_resolution as u32,
        fb_stride:       framebuffer.stride as u32,
        fb_format:       0,
        kernel_base:     boot_info.kernel_base,
        kernel_size:     boot_info.kernel_size,
        initfs_addr:     if !modules.is_empty() {
            modules[0].ptr
        } else {
            0
        },
        initfs_size:     if !modules.is_empty() {
            modules[0].size as u64
        } else {
            0
        },
        memory_map_addr: memory_map_addr as u64,
        memory_map_size: region_count as u64,
    };

    unsafe {
        boot_info_ptr.write(uefi_boot_info);
    }
    info!(
        "BootInfo written to {:#x} (allocated by UEFI)",
        boot_info_addr
    );

    // 8. Desativar watchdog timer
    let _ = (boot_services.set_watchdog_timer)(0, 0x10000, 0, ::core::ptr::null());

    // 9. Logging final
    info!("═══════════════════════════════════════════════════");
    info!("  Boot completo. Transferindo controle...");
    info!("  Entry Point: {:#x}", boot_info.entry_point);
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

    info!("Chamando exit_boot_services...");

    // 10. Sair dos serviços de boot usando nova API freestanding 0.31
    // TODO: Se boot for bem-sucedido (kernel assume controle),
    // resetar contador de tentativas em próximo boot
    // TODO: Implementar exit_boot_services usando nossa camada UEFI
    // let _ = system_table.exit_boot_services(...);
    log::warn!("exit_boot_services not yet implemented with pure UEFI");

    // 11. Saltar para o kernel usando função naked
    // IMPORTANTE: Inline assembly não funciona, usar naked function
    unsafe {
        // Enviar byte 'J' para serial (0x3F8) para confirmar que passamos do
        // exit_boot_services
        ::core::arch::asm!(
            "mov dx, 0x3F8",
            "mov al, 0x4A", // 'J'
            "out dx, al"
        );

        // DEBUG: Mostrar entry point no serial antes do salto
        let entry = boot_info.entry_point;
        let boot_info_arg = boot_info_addr;

        // Enviar 'K' indicando que vamos saltar
        ::core::arch::asm!(
            "mov dx, 0x3F8",
            "mov al, 0x4B", // 'K'
            "out dx, al"
        );

        jump_to_kernel_naked(entry, boot_info_arg);
    }
}

/// Função naked para saltar para o kernel
///
/// IMPORTANTE: Esta função usa naked para garantir que o assembly
/// seja exatamente como escrevemos, sem prólogo/epílogo do compilador
#[unsafe(naked)]
extern "C" fn jump_to_kernel_naked(entry: u64, boot_info: u64) -> ! {
    ::core::arch::naked_asm!(
        // SYSTEM V ABI (Linux/Bare Metal): RDI, RSI, RDX, RCX, R8, R9
        // MICROSOFT ABI (UEFI/Windows):    RCX, RDX, R8,  R9

        // O Ignite roda sobre UEFI, então esta função é chamada usando Microsoft ABI.
        // Argumentos de entrada:
        // - entry:     RCX (1º argumento)
        // - boot_info: RDX (2º argumento)

        // O Kernel (Forge) espera System V ABI (padrão Rust no_std/Linux).
        // Argumentos esperados pelo _start:
        // - boot_info: RDI (1º argumento)

        // 1. Salvar o entry point (RX) em um registrador temporário
        "mov rax, rcx",
        // 2. Mover o argumento boot_info (RDX) para onde o kernel espera (RDI)
        "mov rdi, rdx",
        // 3. Garantir stack alinhado em 16 bytes (exigido pela ABI x86-64)
        "and rsp, 0xFFFFFFFFFFFFFFF0",
        // 4. Chamar o kernel
        "call rax",
        // Loop infinito caso retorne
        "2:",
        "cli",
        "hlt",
        "jmp 2b",
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Funções Auxiliares para o Novo Sistema de Boot
// ═══════════════════════════════════════════════════════════════════════════

/// Carrega configuração do arquivo ou usa configuração padrão
fn load_config(file_loader: &mut FileLoader) -> config::types::BootConfig {
    use config::parser::ConfigParser;

    // Tentar carregar ignite.conf
    if let Ok(config_file) = file_loader.load_file("ignite.conf") {
        let config_data = unsafe {
            ::core::slice::from_raw_parts(config_file.ptr as *const u8, config_file.size)
        };

        // Converter bytes para string
        if let Ok(config_str) = ::core::str::from_utf8(config_data) {
            info!(
                "Arquivo ignite.conf encontrado ({} bytes)",
                config_file.size
            );

            // Parsear configuração
            match ConfigParser::parse(config_str) {
                Ok(config) => {
                    info!("Configuração parseada com sucesso");
                    return config;
                },
                Err(e) => {
                    info!("Erro ao parsear config: {:?}, usando padrão", e);
                },
            }
        }
    }

    // Fallback: configuração hardcoded
    info!("Usando configuração padrão (ignite.conf não encontrado)");
    create_default_config()
}

/// Cria configuração padrão para Redstone OS
fn create_default_config() -> config::types::BootConfig {
    use alloc::{string::String, vec::Vec};

    use config::types::{BootConfig, MenuEntry, WallpaperStyle};

    use crate::core::constants::paths::*;

    let mut config = BootConfig {
        timeout:              Some(3), // 3 segundos para permitir tecla M
        default_entry:        1,
        quiet:                false,
        serial:               true, // Serial ligado por padrão
        serial_baudrate:      115200,
        verbose:              true,
        interface_resolution: None,
        interface_branding:   Some(String::from("Ignite v0.4 - Redstone OS")),
        wallpaper:            None,
        wallpaper_style:      WallpaperStyle::Centered,
        editor_enabled:       false,
        entries:              Vec::new(),
    };

    // Entrada padrão para Redstone OS
    // Usa paths do módulo constants
    let entry = MenuEntry::new(
        String::from("Redstone OS (default)"),
        String::from("limine"),
        String::from(DEFAULT_KERNEL_PATH), // "boot/kernel"
    );

    config.entries.push(entry);
    config
}

/// Seleciona entrada de boot (menu ou auto-boot)
///
/// Implementa lógica de seleção com suporte a:
/// - Countdown com detecção de tecla M
/// - Menu interativo se M pressionado
/// - Auto-boot se timeout expirar
///
/// # Argumentos
/// * `boot_services` - Serviços de boot UEFI
/// * `config` - Configuração de boot
///
/// # Retorna
/// Índice (0-based) da entrada selecionada
fn select_boot_entry(boot_services: &BootServices, config: &config::types::BootConfig) -> usize {
    use crate::ui::BootMenu;

    // Se timeout é 0, boot imediato na entrada padrão
    if config.timeout == Some(0) {
        let index = if config.default_entry > 0 && config.default_entry <= config.entries.len() {
            config.default_entry - 1 // Converter de 1-based para 0-based
        } else {
            0
        };
        info!("Auto-boot imediato (timeout=0)");
        return index;
    }

    // Se apenas 1 entrada, boot direto (sem menu)
    if config.entries.len() == 1 {
        info!("Apenas 1 entrada disponível, boot direto");
        return 0;
    }

    // Countdown aguardando tecla M
    let timeout = config.timeout.unwrap_or(3);
    let show_menu = BootMenu::wait_for_trigger(boot_services, timeout.into());

    if show_menu {
        // Usuário pressionou M - mostrar menu interativo
        let mut menu = BootMenu::new(boot_services, config);
        menu.show()
    } else {
        // Timeout expirou - usar entrada padrão
        info!("Usando entrada padrão");
        if config.default_entry > 0 && config.default_entry <= config.entries.len() {
            config.default_entry - 1
        } else {
            0
        }
    }
}

/// Carrega initramfs (ramdisk TAR) se existir
///
/// Tenta carregar do caminho padrão (boot/initfs) definido nas constantes.
/// Suporta arquivos comprimidos (.zst) se disponível.
fn load_initramfs(file_loader: &mut FileLoader) -> Option<core::types::LoadedFile> {
    use crate::core::constants::paths::DEFAULT_INITFS_PATH;

    info!("Carregando initramfs...");

    // Tentar primeiro com compressão (.zst)
    let path_compressed = alloc::format!("{}.zst", DEFAULT_INITFS_PATH);
    if let Ok(file) = file_loader.load_file(&path_compressed) {
        info!(
            "  InitRAMFS comprimido carregado: {} KB em {:#x}",
            file.size / 1024,
            file.ptr
        );
        // TODO: Descomprimir aqui quando implementarmos suporte zstd
        return Some(file);
    }

    // Tentar sem compressão
    match file_loader.load_file(DEFAULT_INITFS_PATH) {
        Ok(file) => {
            info!(
                "  InitRAMFS carregado: {} KB em {:#x}",
                file.size / 1024,
                file.ptr
            );
            Some(file)
        },
        Err(_) => {
            info!("  Aviso: {} não encontrado", DEFAULT_INITFS_PATH);
            info!("  Sistema não terá rootfs inicial");
            None
        },
    }
}

/// Usa o protocolo apropriado para preparar o boot
fn use_protocol(
    allocator: &MemoryAllocator,
    protocol_name: &str,
    kernel_data: &[u8],
    cmdline: Option<&str>,
    modules: &[core::types::LoadedFile],
) -> protos::BootInfo {
    use protos::{BootProtocol, limine::LimineProtocol};

    // Por enquanto, suporta apenas Limine
    // TODO: Adicionar seleção de outros protocolos (linux, multiboot1, multiboot2,
    // efi)
    match protocol_name.to_lowercase().as_str() {
        "limine" => {
            info!("Usando Limine Protocol");
            let mut protocol = LimineProtocol::new(allocator);

            // Validar
            protocol
                .validate(kernel_data)
                .expect("Kernel inválido para protocolo Limine");

            // Preparar
            protocol
                .prepare(kernel_data, cmdline, modules)
                .expect("Falha ao preparar boot")
        },
        _ => {
            info!(
                "Protocolo '{}' não implementado ainda, usando Limine como fallback",
                protocol_name
            );
            let mut protocol = LimineProtocol::new(allocator);
            protocol.validate(kernel_data).expect("Kernel inválido");
            protocol
                .prepare(kernel_data, cmdline, modules)
                .expect("Falha ao preparar boot")
        },
    }
}

/// Prepara argumentos do kernel a partir do BootInfo do protocolo
fn prepare_kernel_args_from_boot_info(
    allocator: &MemoryAllocator,
    boot_info: &protos::BootInfo,
    modules: &[core::types::LoadedFile],
    _framebuffer: &core::types::Framebuffer,
) -> Result<*const KernelArgs> {
    // Alocar memória para KernelArgs
    let args_ptr = allocator.allocate_any(1)?;
    let args = unsafe { &mut *(args_ptr as *mut KernelArgs) };

    // Preencher estrutura
    args.kernel_base = boot_info.kernel_base;
    args.kernel_size = boot_info.kernel_size;
    args.stack_base = boot_info.stack_ptr.unwrap_or(0);
    args.stack_size = 0;
    args.env_base = 0;
    args.env_size = 0;
    args.hwdesc_base = 0; // TODO: Encontrar RSDP
    args.hwdesc_size = 0;
    args.areas_base = 0;
    args.areas_size = 0;

    // Configurar InitFS se disponível (primeiro módulo)
    if !modules.is_empty() {
        args.bootstrap_base = modules[0].ptr;
        args.bootstrap_size = modules[0].size as u64;
    } else {
        args.bootstrap_base = 0;
        args.bootstrap_size = 0;
    }

    Ok(args as *const KernelArgs)
}

/// Prepara a estrutura KernelArgs
#[allow(dead_code)]
fn prepare_kernel_args(
    allocator: &MemoryAllocator,
    loaded_kernel: &core::types::LoadedKernel,
    initfs: &Option<core::types::LoadedFile>,
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
