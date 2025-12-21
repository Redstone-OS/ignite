use core::ptr;

use log::error;
use uefi::Status;

use crate::os::{OsVideoMode, uefi::display::Output};

pub struct VideoModeIter {
    output_opt: Option<Output>,
    i:          usize,
}

impl VideoModeIter {
    pub fn new(output_opt: Option<Output>) -> Self {
        Self { output_opt, i: 0 }
    }
}

impl Iterator for VideoModeIter {
    type Item = OsVideoMode;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut output) = self.output_opt {
            let st = uefi::table::system_table_boot().unwrap();
            let bs = st.boot_services();
            // output.0 é GraphicsOutput. modes() aceita &BootServices.
            // Checar assinatura.
            // Precisamos avançar para self.i
            // Já que o iterador de modo consome, reconstruímos ele toda vez? Ineficiente
            // mas seguro. Ou apenas usar nth(self.i).

            let st = uefi::table::system_table_boot().unwrap();
            let bs = st.boot_services();
            if let Some(mode) = output.0.modes(bs).nth(self.i) {
                let info = mode.info();
                let id = self.i;
                self.i += 1;

                return Some(OsVideoMode {
                    id:     id as u32, // ID deve ser o índice
                    width:  info.resolution().0 as u32,
                    height: info.resolution().1 as u32,
                    stride: info.stride() as u32,
                    base:   0,
                });
            }
        }
        None
    }
}
