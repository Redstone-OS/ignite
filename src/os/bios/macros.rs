/// Imprimir para o console
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        #[cfg(feature = "serial_debug")]
        {
            let _ = write!($crate::os::serial::COM1.lock(), $($arg)*);
        }
        let _ = write!($crate::os::VGA.lock(), $($arg)*);
    });
}

/// Imprimir com nova linha para o console
#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr_2021) => (print!(concat!($fmt, "\n")));
    ($fmt:expr_2021, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}
