use core::slice;

use crate::{
    area_add,
    os::{Os, OsMemoryEntry, OsMemoryKind},
};

const ENTRY_ADDRESS_MASK: u64 = 0x000F_FFFF_FFFF_F000;
const PAGE_ENTRIES: usize = 512;
const PAGE_SIZE: usize = 4096;
pub(crate) const PHYS_OFFSET: u64 = 0xFFFF_8000_0000_0000;

unsafe fn paging_allocate(os: &impl Os) -> Option<&'static mut [u64]> {
    unsafe {
        let ptr = os.alloc_zeroed_page_aligned(PAGE_SIZE);
        if !ptr.is_null() {
            area_add(OsMemoryEntry {
                base: ptr as u64,
                size: PAGE_SIZE as u64,
                kind: OsMemoryKind::Reclaim,
            });

            Some(slice::from_raw_parts_mut(ptr as *mut u64, PAGE_ENTRIES))
        } else {
            None
        }
    }
}

const PRESENT: u64 = 1;
const WRITABLE: u64 = 1 << 1;
const LARGE: u64 = 1 << 7;

pub unsafe fn paging_create(os: &impl Os, kernel_phys: u64, kernel_size: u64) -> Option<usize> {
    unsafe {
        // Criar PML4
        let pml4 = paging_allocate(os)?;

        {
            // Criar PDP para mapeamento de identidade
            let pdp = paging_allocate(os)?;

            // Linkar primeira entrada PML4 de usuário e primeira de kernel para PDP
            pml4[0] = pdp.as_ptr() as u64 | WRITABLE | PRESENT;
            pml4[256] = pdp.as_ptr() as u64 | WRITABLE | PRESENT;

            // Mapeamento de identidade de 8 GiB usando páginas de 2 MiB
            for pdp_i in 0..8 {
                let pd = paging_allocate(os)?;
                pdp[pdp_i] = pd.as_ptr() as u64 | WRITABLE | PRESENT;
                for pd_i in 0..pd.len() {
                    let addr = pdp_i as u64 * 0x4000_0000 + pd_i as u64 * 0x20_0000;
                    pd[pd_i] = addr | LARGE | WRITABLE | PRESENT;
                }
            }
        }

        {
            // Criar PDP (abrangendo 512 GiB) para mapeamento do kernel
            let pdp = paging_allocate(os)?;

            // Linkar última entrada PML4 para PDP
            pml4[511] = pdp.as_ptr() as u64 | WRITABLE | PRESENT;

            // Criar PD (abrangendo 1 GiB) para mapeamento do kernel.
            let pd = paging_allocate(os)?;

            // O kernel é mapeado em -2^31, ou seja, 0xFFFF_FFFF_8000_0000. Como um PD é 1
            // GiB, linkar a penúltima entrada PDP para PD.
            pdp[510] = pd.as_ptr() as u64 | WRITABLE | PRESENT;

            // Mapear kernel_size bytes no offset do kernel, ou seja, para o início do PD.

            let mut kernel_mapped = 0;

            let mut pd_idx = 0;
            while kernel_mapped < kernel_size && pd_idx < pd.len() {
                let pt = paging_allocate(os)?;
                pd[pd_idx] = pt.as_ptr() as u64 | WRITABLE | PRESENT;
                pd_idx += 1;

                let mut pt_idx = 0;
                while kernel_mapped < kernel_size && pt_idx < pt.len() {
                    let addr = kernel_phys + kernel_mapped;
                    pt[pt_idx] = addr | WRITABLE | PRESENT;
                    pt_idx += 1;
                    kernel_mapped += PAGE_SIZE as u64;
                }
            }
            assert!(kernel_mapped >= kernel_size);
        }

        Some(pml4.as_ptr() as usize)
    }
}

pub unsafe fn paging_framebuffer(
    os: &impl Os,
    page_phys: usize,
    framebuffer_phys: u64,
    framebuffer_size: u64,
) -> Option<u64> {
    unsafe {
        // TODO: teste mais inteligente para framebuffer já mapeado
        if framebuffer_phys + framebuffer_size <= 0x2_0000_0000 {
            return Some(framebuffer_phys + PHYS_OFFSET);
        }

        let pml4_i = ((framebuffer_phys / 0x80_0000_0000) + 256) as usize;
        let mut pdp_i = ((framebuffer_phys % 0x80_0000_0000) / 0x4000_0000) as usize;
        let mut pd_i = ((framebuffer_phys % 0x4000_0000) / 0x20_0000) as usize;
        assert_eq!(framebuffer_phys % 0x20_0000, 0);

        let pml4 = slice::from_raw_parts_mut(page_phys as *mut u64, PAGE_ENTRIES);

        // Criar PDP para mapeamento de framebuffer
        let pdp = if pml4[pml4_i] == 0 {
            let pdp = paging_allocate(os)?;
            pml4[pml4_i] = pdp.as_ptr() as u64 | 1 << 1 | 1;
            pdp
        } else {
            slice::from_raw_parts_mut(
                (pml4[pml4_i] & ENTRY_ADDRESS_MASK) as *mut u64,
                PAGE_ENTRIES,
            )
        };

        // Mapear framebuffer_size no offset do framebuffer
        let mut framebuffer_mapped = 0;
        while framebuffer_mapped < framebuffer_size && pdp_i < pdp.len() {
            let pd = paging_allocate(os)?;
            assert_eq!(pdp[pdp_i], 0);
            pdp[pdp_i] = pd.as_ptr() as u64 | 1 << 1 | 1;

            while framebuffer_mapped < framebuffer_size && pd_i < pd.len() {
                let addr = framebuffer_phys + framebuffer_mapped;
                assert_eq!(pd[pd_i], 0);
                pd[pd_i] = addr | 1 << 7 | 1 << 1 | 1;
                framebuffer_mapped += 0x20_0000;
                pd_i += 1;
            }

            pdp_i += 1;
            pd_i = 0;
        }
        assert!(framebuffer_mapped >= framebuffer_size);

        Some(framebuffer_phys + PHYS_OFFSET)
    }
}
