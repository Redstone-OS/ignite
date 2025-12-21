//! Helper implementations for BootServices
//!
//! Fornece métodos convenientes sobre os FFI function pointers

use super::boot::*;
use crate::uefi::base::*;

impl BootServices {
    /// Aloca páginas de memória (wrapper conveniente)
    pub unsafe fn allocate_pages_helper(
        &self,
        ty: AllocateType,
        mem_ty: MemoryType,
        pages: usize,
    ) -> Result<u64> {
        let mut addr: u64 = match ty {
            AllocateType::AllocateAnyPages => 0,
            AllocateType::AllocateMaxAddress => 0,
            AllocateType::AllocateAddress => 0,
            _ => 0,
        };

        let status = (self.allocate_pages)(ty, mem_ty, pages, &mut addr);
        status.to_result_with_val(addr)
    }

    /// Libera páginas de memória (wrapper conveniente)
    pub unsafe fn free_pages_helper(&self, addr: u64, pages: usize) -> Result<()> {
        let status = (self.free_pages)(addr, pages);
        status.to_result()
    }

    /// Aguarda por um período (microsegundos)
    pub fn stall(&self, microseconds: usize) -> Result<()> {
        let status = unsafe { (self.stall)(microseconds) };
        status.to_result()
    }
}
