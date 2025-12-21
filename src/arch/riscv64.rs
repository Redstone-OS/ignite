//! Suporte a RISC-V 64 - Placeholder
//!
//! TODO: Implementar suporte a placas RISC-V (ex: VisionFive 2).

pub fn init() {
    // No-op for now
}

pub fn hlt() {
    unsafe {
        core::arch::asm!("wfi"); // Wait For Interrupt
    }
}
