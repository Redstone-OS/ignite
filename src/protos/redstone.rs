//! Protocolo Nativo Redstone
//! ------------------------
//!
//! Implementa um `BootProtocol` simples e robusto para kernels ELF64 no
//! ecossistema Redstone. Este módulo é responsável por:
//!
//! - Identificar se um binário é um ELF64 inicializável.
//! - Preparar mapeamentos de memória essenciais (identity map + scratch slot).
//! - Carregar segmentos ELF do kernel para a memória apropriada.
//! - Construir e gravar uma estrutura `BootInfo` que será passada ao kernel.
//!
//! ## Invariantes críticas de inicialização
//!
//! 1. **Identity map dos primeiros 4 GiB** deve ser criado antes de carregar o
//!    kernel.
//!    - Garante acesso seguro a memoria física baixa, frequentemente usada pelo
//!      kernel durante bootstrap (zerar frames, criar page tables, etc).
//!    - Usa huge pages (2 MiB) para reduzir TLB pressure e acelerar
//!      mapeamentos.
//!
//! 2. **Scratch slot** deve existir **após** o identity map e **antes** do
//!    carregamento do kernel.
//!    - É uma região virtual dedicada que o kernel poderá usar para mapear
//!      frames físicos temporariamente enquanto inicializa suas próprias page
//!      tables.
//!    - Deve ser alocada em região livre de conflitos com os mapeamentos de
//!      huge pages.
//!
//! 3. **Carregar o kernel** somente depois das etapas anteriores.
//!    - Dessa forma, qualquer operação do kernel que dependa de mapeamentos
//!      físicos ou do scratch slot não causará General Protection Fault (GPF).
//!
//! Estas ordens são **essenciais** — violá-las é a causa mais comum de panics e
//! GPFs durante o early bootstrap. Se você estiver vendo faults ao escrever
//! frames baixos, verifique o cumprimento dessas três etapas na ordem exata
//! descrita aqui.
//!
//! ## Extensões e TODOs óbvios
//! - Leitura do RSDP/ACPI para preencher `rsdp_addr` em `BootInfo`.
//! - Implementar `prepare_framebuffer()` real que consulta o firmware/UEFI para
//!   obter `addr`, `width`, `height`, `stride` e `format` reais.
//! - Suporte a múltiplos módulos (initramfs + módulos adicionais) e validação
//!   do conteúdo.
//! - Migração de panics (`expect`) para tratamento de erro robusto e propagação
//!   com `Result` (dependendo das variantes de `BootError` do crate).
//!
//! ## Nota de segurança
//! Código de boot roda em contexto privilegiado e com pouco suporte de runtime.
//! Comentários abaixo explicam por que certas operações (escrever diretamente
//! ponteiros físicos, `unsafe`) são necessárias e quais invariantes devem ser
//! respeitadas ao modificá-las.
//!
//! ------------------------------------------------------------------------------

use alloc::vec::Vec;

use super::{BootProtocol, KernelLaunchInfo};
use crate::{
    core::{
        error::{BootError, Result},
        handoff::{BootInfo, FramebufferInfo, MemoryInfo},
        types::LoadedFile,
    },
    elf::ElfLoader,
    memory::{FrameAllocator, PageTableManager},
    uefi::system_table,
};

/// Implementa o protocolo de boot "nativo" do Redstone.
///
/// `RedstoneProtocol` encapsula os recursos necessários para preparar o
/// ambiente de execução do kernel (página, frames, loader ELF) e construir o
/// `BootInfo`.
///
/// **Responsabilidades principais**
/// - Garantir os mapeamentos de memória exigidos pelo kernel (identity map +
///   scratch).
/// - Carregar segmentos ELF e produzir `KernelLaunchInfo` contendo entry point
///   e ponteiros necessários (ex.: ponteiro físico para `BootInfo` em `rdi`).
///
/// **Observações de design**
/// - O `allocator` e `page_table` são referências mutáveis externas: este
///   objeto não possui ownership sobre a memória física — ele apenas orquestra
///   as operações.
/// - Muitas operações de baixo nível são `unsafe` por natureza (escrever
///   estruturas diretamente em memória física); mantenha as invariantes e
///   documente TODOs.
pub struct RedstoneProtocol<'a> {
    allocator:  &'a mut dyn FrameAllocator,
    page_table: &'a mut PageTableManager,
}

impl<'a> RedstoneProtocol<'a> {
    /// Cria uma nova instância do protocolo.
    ///
    /// # Parâmetros
    /// - `allocator`: alocador de frames físicos utilizado para reservar
    ///   `BootInfo` e outras estruturas temporárias.
    /// - `page_table`: gerenciador de page tables usado para criar identity
    ///   maps e o scratch slot.
    ///
    /// A função assume que `allocator` e `page_table` permanecem válidos
    /// durante todo o lifetime do protocolo. Não faz cópia dos objetos;
    /// apenas guarda referências.
    pub fn new(
        allocator: &'a mut dyn FrameAllocator,
        page_table: &'a mut PageTableManager,
    ) -> Self {
        Self {
            allocator,
            page_table,
        }
    }

    /// Prepara informações do framebuffer.
    ///
    /// Atualmente é um *stub seguro* que retorna um `FramebufferInfo` neutro.
    /// Deve ser substituído por uma implementação que:
    ///  - consulte o firmware/UEFI (ex.: `system_table`),
    ///  - valide se o framebuffer é linear e mapeável,
    ///  - preencha `addr`, `size`, `width`, `height`, `stride`, `format`.
    ///
    /// Enquanto isso, retornamos valores nulos coerentes para evitar
    /// comportamentos indefinidos no kernel quando o framebuffer não
    /// estiver disponível.
    fn prepare_framebuffer(&self) -> FramebufferInfo {
        // Stub seguro — evita passar lixo para o kernel.
        FramebufferInfo {
            addr:   0,
            size:   0,
            width:  0,
            height: 0,
            stride: 0,
            format: crate::core::handoff::PixelFormat::Rgb,
        }
    }
}

impl<'a> BootProtocol for RedstoneProtocol<'a> {
    /// Nome do protocolo — usado para logs/diagnóstico.
    fn name(&self) -> &str {
        "Redstone Native"
    }

    /// Identifica se `file_content` parece ser um ELF.
    ///
    /// Critério simples: verifica magic ELF (`0x7F, 'E', 'L', 'F'`).
    /// Esta função é propositalmente minimalista; validadores adicionais
    /// (classe ELF, endianness, ABI, tipo de máquina) podem ser adicionados se
    /// necessário para rejeitar binários incompatíveis.
    fn identify(&self, file_content: &[u8]) -> bool {
        // Verifica Magic ELF: 0x7F 'E' 'L' 'F'
        file_content.len() > 4 && &file_content[0..4] == b"\x7fELF"
    }

    /// Processo principal de carregamento do kernel + criação do `BootInfo`.
    ///
    /// Fluxo resumido:
    /// 1. Criar *identity map* para os primeiros 4 GiB usando huge pages de 2
    ///    MiB.
    /// 2. Criar o *scratch slot* (região virtual temporária para operações do
    ///    kernel).
    /// 3. Usar `ElfLoader` para interpretar e mapear segmentos ELF do
    ///    `kernel_file`.
    /// 4. Alocar um frame físico para a estrutura `BootInfo`.
    /// 5. Preencher e escrever `BootInfo` na memória física alocada.
    /// 6. Retornar `KernelLaunchInfo` com entry point e argumentos de boot.
    ///
    /// **Parâmetros**
    /// - `kernel_file`: bytes do binário do kernel (ELF).
    /// - `_cmdline`: linha de comando (atualmente não utilizada).
    /// - `modules`: lista de módulos anexados (primeiro modul é tratado como
    ///   initrd).
    /// - `memory_map_buffer`: tupla `(addr, len)` apontando para o buffer do
    ///   mapa de memória.
    ///
    /// **Retorno**
    /// - `Ok(KernelLaunchInfo)`: sucesso — inclui `entry_point` e `rdi`
    ///   apontando para `BootInfo`.
    /// - `Err(BootError)`: falha ao carregar ou alocar recursos.
    fn load(
        &mut self,
        kernel_file: &[u8],
        _cmdline: Option<&str>,
        modules: Vec<LoadedFile>,
        memory_map_buffer: (u64, u64),
    ) -> Result<KernelLaunchInfo> {
        // ---------------------------
        // 1) Identity map dos primeiros 4GiB
        // ---------------------------
        //
        // **Por que:** muitos kernels esperam que a memória física baixa seja acessível
        // durante a fase inicial (por exemplo, ao zerar frames ou construir page
        // tables).
        //
        // NOTA: Aqui usamos `expect` na falha pois, no contexto de boot, falha em criar
        // identity map normalmente é uma condição fatal. Idealmente migrar para erro
        // retornado (por exemplo, `map_err(|e| BootError::MapFailure(e))`) quando o
        // enum `BootError` suportar variantes apropriadas.
        self.page_table
            .identity_map_4gib(self.allocator)
            .expect("Falha ao criar identity map (4GiB)");

        // ---------------------------
        // 2) Carregar segmentos ELF do kernel
        // ---------------------------
        //
        // **IMPORTANTE:** O kernel deve ser carregado ANTES do scratch slot!
        // Isso garante que a memória ocupada pelo kernel não sobreponha
        // as page tables do scratch slot que serão alocadas a seguir.
        //
        // `ElfLoader` é responsável por interpretar os headers ELF e mapear (ou copiar)
        // os segmentos para o espaço físico apropriado. O loader deve retornar:
        //   - base_address (físico) do kernel
        //   - entry_point virtual/relativo (dependendo do layout)
        //   - tamanho total carregado
        //
        // Se o kernel requer relocation/relro/relro-fixups, o loader é o local correto
        // para aplicar essas transformações.
        let mut loader = ElfLoader::new(self.allocator, self.page_table);
        let loaded_kernel = loader.load_kernel(kernel_file)?;

        // ---------------------------
        // 3) Configurar scratch slot para o kernel
        // ---------------------------
        //
        // O scratch slot é uma região virtual que o kernel usa temporariamente para
        // mapear frames físicos. DEVE ser alocado APÓS o kernel para evitar que
        // o kernel sobrescreva as page tables do scratch!
        self.page_table
            .setup_scratch_slot(self.allocator)
            .expect("Falha ao configurar scratch slot");

        // ---------------------------
        // 4) Alocar BootInfo (frame físico)
        // ---------------------------
        //
        // Reservamos um frame físico (1 frame = 4 KiB, presumivelmente) para escrever a
        // estrutura `BootInfo`. Em seguida passamos o endereço físico deste frame no
        // registro `rdi` (convenção escolhida pelo protocolo Redstone).
        let boot_info_phys = self.allocator.allocate_frame(1)?;
        let boot_info_ptr = boot_info_phys as *mut BootInfo;

        // ---------------------------
        // 5) Preencher BootInfo
        // ---------------------------
        //
        // Montamos os campos conhecidos — framebuffer, mapa de memória, kernel infos,
        // initrd.
        let fb_info = self.prepare_framebuffer();

        // Tratamos o primeiro módulo como initrd, se presente. Em futuros updates:
        // - suportar múltiplos módulos com uma lista em BootInfo,
        // - validar assinaturas/hashe(s) do initrd,
        // - garantir alinhamento do initrd em páginas.
        let (initrd_addr, initrd_size) = if let Some(first_mod) = modules.first() {
            (first_mod.ptr, first_mod.size as u64)
        } else {
            (0, 0)
        };

        let boot_info = BootInfo {
            // Versão/magic para validação pelo kernel.
            magic:   crate::core::handoff::BOOT_INFO_MAGIC,
            version: crate::core::handoff::BOOT_INFO_VERSION,

            framebuffer: fb_info,

            // Ponteiro e comprimento das entradas do memory map (fornecido pelo firmware/loader).
            memory_map_addr: memory_map_buffer.0,
            memory_map_len:  memory_map_buffer.1,

            // ACPI RSDP — preenchido futuramente.
            rsdp_addr: 0,

            // Informações fundamentais do kernel carregado.
            kernel_phys_addr: loaded_kernel.base_address,
            kernel_size:      loaded_kernel.size,

            // Initramfs (initrd) — se houver.
            initramfs_addr: initrd_addr,
            initramfs_size: initrd_size,
        };

        // ---------------------------
        // 6) Escrever BootInfo no frame alocado
        // ---------------------------
        //
        // Segurança: escrever em memória física requer `unsafe`. Garantimos:
        // - `boot_info_ptr` aponta a um frame válido maior que sizeof(BootInfo).
        // - BootInfo é `Copy`/plain-old-data (ou ao menos consistência de layout).
        //
        // Se for necessário limpar/validar a página antes de escrever, este é o lugar.
        unsafe {
            core::ptr::write(boot_info_ptr, boot_info);
        }

        // ---------------------------
        // 7) Construir KernelLaunchInfo e retornar
        // ---------------------------
        //
        // `use_fixed_redstone_entry = true` indica que o protocolo espera executar um
        // entry jump fixo no loader do Redstone. Registradores RDI/RSI/.. são definidos
        // conforme contrato do handoff.
        Ok(KernelLaunchInfo {
            entry_point: loaded_kernel.entry_point,
            use_fixed_redstone_entry: true,
            stack_pointer: None,
            rdi: boot_info_phys,
            rsi: 0,
            rdx: 0,
            rbx: 0,
        })
    }
}
