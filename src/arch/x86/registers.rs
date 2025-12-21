//! Manipulação de Registradores de Controle (CRs) e MSRs
//!
//! Necessário para configurar Paginação (CR3) e Modos de CPU (CR0/CR4).

/// Lê o valor atual do registrador CR3 (Page Map Level 4 Base).
#[inline]
pub fn read_cr3() -> u64 {
    let value: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

/// Escreve um novo valor no registrador CR3.
/// Isso troca o espaço de endereçamento virtual ativo.
///
/// # Safety
/// O chamador deve garantir que `value` aponta para uma PML4 válida na memória
/// física.
#[inline]
pub unsafe fn write_cr3(value: u64) {
    core::arch::asm!("mov cr3, {}", in(reg) value, options(nomem, nostack, preserves_flags));
}

/// Invalida a TLB para um endereço específico (INVLPG).
/// Deve ser chamado ao alterar mapeamentos de página.
#[inline]
pub unsafe fn flush_tlb(addr: u64) {
    core::arch::asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
}

/// Lê o registrador RFLAGS.
#[inline]
pub fn read_rflags() -> u64 {
    let r: u64;
    unsafe {
        core::arch::asm!("pushfq; pop {}", out(reg) r, options(nomem, preserves_flags));
    }
    r
}
