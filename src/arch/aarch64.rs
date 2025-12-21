//! Suporte a ARM64 (AArch64) - Placeholder
//!
//! TODO: Implementar quando o suporte a placas ARM (ex: Raspberry Pi) for
//! necessário.

pub fn init() {
    // No-op for now
}

pub fn hlt() {
    unsafe {
        core::arch::asm!("wfi"); // Wait For Interrupt
    }
}
