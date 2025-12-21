//! Implementações helper para BootServices

use super::boot::*;
use crate::uefi::base::*;

impl BootServices {
    /// Aloca páginas de memória (wrapper conveniente sobre FFI)
    #[inline]
    pub unsafe fn allocate_pages_helper(
        &self,
        ty: AllocateType,
        mem_ty: MemoryType,
        pages: usize,
    ) -> Result<u64> {
        let mut addr: u64 = 0;
        let status = (self.allocate_pages)(ty, mem_ty, pages, &mut addr);
        status.to_result_with_val(addr)
    }

    /// Libera páginas de memória (wrapper conveniente sobre FFI)
    #[inline]
    pub unsafe fn free_pages_helper(&self, addr: u64, pages: usize) -> Result<()> {
        let status = (self.free_pages)(addr, pages);
        status.to_result()
    }

    /// Aguarda por um período (microsegundos)
    #[inline]
    pub fn stall_helper(&self, microseconds: usize) -> Result<()> {
        let status = (self.stall)(microseconds);
        status.to_result()
    }

    /// Acessa o console stdin from SystemTable
    ///
    /// NOTA: Este método assume que o BootServices foi obtido de uma
    /// SystemTable e usa offset para encontrar SystemTable. Em produção,
    /// passar SystemTable explicitamente.
    #[inline]
    pub fn stdin(&self) -> *mut crate::uefi::table::system::SimpleTextInputProtocol {
        // Por enquanto retorna null - requer refatoração para passar SystemTable
        // TODO: Passar SystemTable como parâmetro ou armazenar referência
        core::ptr::null_mut()
    }

    /// Acessa o console stdout from SystemTable
    #[inline]
    pub fn stdout(&self) -> *mut crate::uefi::table::system::SimpleTextOutputProtocol {
        // Por enquanto retorna null - requer refatoração para passar SystemTable
        // TODO: Passar SystemTable como parâmetro ou armazenar referência
        core::ptr::null_mut()
    }
}
