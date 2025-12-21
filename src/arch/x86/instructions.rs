//! Instruções Especiais da CPU

/// Para a execução da CPU até a próxima interrupção.
/// Usado no loop idle para economizar energia.
#[inline]
pub fn hlt() {
    unsafe {
        core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

/// Dica para a CPU que estamos em um spinloop.
/// Melhora performance em Hyper-Threading e economiza energia.
#[inline]
pub fn pause() {
    unsafe {
        core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
    }
}

/// Desabilita interrupções (CLI).
#[inline]
pub unsafe fn disable_interrupts() {
    core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
}

/// Habilita interrupções (STI).
#[inline]
pub unsafe fn enable_interrupts() {
    core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
}

/// Breakpoint de software (INT 3).
#[inline]
pub fn debug_break() {
    unsafe {
        core::arch::asm!("int3", options(nomem, nostack, preserves_flags));
    }
}
