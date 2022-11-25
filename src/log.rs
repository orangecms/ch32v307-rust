//! Log system for BT0
// essentially copied from sunxi/nezha

// TODO
// use crate::init::Serial;
use core::fmt;
use embedded_hal::serial::nb::Write;
use nb::block;

#[derive(Debug)]
pub struct Serial<'a> {
    uart: &'a ch32v3::ch32v30x::USART1,
}

/// Error types that may happen when serial transfer
#[derive(Debug)]
pub struct Error {
    kind: embedded_hal::serial::ErrorKind,
}

impl embedded_hal::serial::Error for Error {
    #[inline]
    fn kind(&self) -> embedded_hal::serial::ErrorKind {
        self.kind
    }
}

impl Serial<'_> {
    #[inline]
    pub fn new(uart: &ch32v3::ch32v30x::USART1) -> Self {
        // TODO
        // uart_init();
        Self { uart }
    }
}

impl embedded_hal::serial::ErrorType for Serial<'_> {
    type Error = Error;
}

impl embedded_hal::serial::nb::Write<u8> for Serial<'_> {
    #[inline]
    fn write(&mut self, c: u8) -> nb::Result<(), self::Error> {
        if self.uart.statr.read().txe() != true {
            return Err(nb::Error::WouldBlock);
        }
        unsafe {
            self.uart.datar.modify(|_, w| w.bits(c as u32));
        }
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> nb::Result<(), self::Error> {
        // TODO
        let TFE_EMPTY = true;
        if TFE_EMPTY {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

type S<'a> = Wrap<Serial<'a>>;

#[doc(hidden)]
pub(crate) static mut LOGGER: Option<Logger> = None;

// type `Serial` is declared outside this crate, avoid orphan rule
pub(crate) struct Wrap<T>(T);

#[doc(hidden)]
pub(crate) struct Logger<'a> {
    pub(crate) inner: S<'a>,
}

impl fmt::Write for S<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for byte in s.as_bytes() {
            block!(self.0.write(*byte)).unwrap();
        }
        block!(self.0.flush()).unwrap();
        Ok(())
    }
}

#[inline]
pub fn set_logger(serial: Serial) {
    unsafe {
        LOGGER = Some(Logger {
            inner: Wrap(serial),
        });
    }
}

#[inline]
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    unsafe {
        match &mut LOGGER {
            Some(l) => l.inner.write_fmt(args).unwrap(),
            _ => {}
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::log::_print(core::format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($($arg:tt)*) => {
        $crate::log::_print(core::format_args!($($arg)*));
        $crate::print!("\r\n");
    }
}
