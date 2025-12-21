//! Cabeçalho Padrão de Tabelas UEFI
//!
//! Todas as tabelas principais (System, Boot, Runtime) começam com esta
//! estrutura. Referência: UEFI Spec 2.10, Seção 4.2

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TableHeader {
    /// Assinatura única da tabela (ex: "IBI SYST" para System Table).
    pub signature:   u64,
    /// Versão da especificação ou da tabela.
    pub revision:    u32,
    /// Tamanho do cabeçalho em bytes.
    pub header_size: u32,
    /// CRC32 da tabela (deve ser 0 durante o cálculo).
    pub crc32:       u32,
    /// Reservado (deve ser 0).
    pub reserved:    u32,
}
