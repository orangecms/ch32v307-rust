//! Log system for BT0
// essentially copied from sunxi/nezha

// TODO
// use crate::init::Serial;
use core::fmt;
use embedded_hal::serial::nb::Write;
use nb::block;

#[derive(Debug)]
pub struct Serial {
    uart: ch32v3::ch32v30x::USART1,
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

impl Serial {
    #[inline]
    pub fn new(uart: ch32v3::ch32v30x::USART1) -> Self {
        // enable this UART, set word length (m) to 8 bits
        uart.ctlr1.modify(|_, w| w.ue().set_bit().m().clear_bit());
        // enable transmitter and its interrupt (TX empty)
        uart.ctlr1.modify(|_, w| w.te().set_bit().txeie().set_bit());
        // enable receiver and its interrupt (RX non-empty)
        uart.ctlr1.modify(|_, w| w.re().set_bit().rxneie().set_bit());
        unsafe {
            // 1 stop bit
            uart.ctlr2.modify(|_, w| w.stop().bits(0b00));
            // 12 bits mantissa, last 4 bits are fraction (1/16)
            uart.brr.modify(|_, w| w.div_mantissa().bits(39).div_fraction().bits(1));
        }
        Self { uart }
    }
}

impl embedded_hal::serial::ErrorType for Serial {
    type Error = Error;
}

impl embedded_hal::serial::nb::Read<u8> for Serial {
    fn read(&mut self) -> nb::Result<u8, self::Error> {
        if self.uart.statr.read().rxne() != true {
            return Err(nb::Error::WouldBlock);
        }
        unsafe {
            Ok(self.uart.datar.read().bits() as u8)
        }
    }
}

impl embedded_hal::serial::nb::Write<u8> for Serial {
    fn write(&mut self, c: u8) -> nb::Result<(), self::Error> {
        if self.uart.statr.read().txe() != true {
            return Err(nb::Error::WouldBlock);
        }
        unsafe {
            self.uart.datar.write(|w| w.bits(c as u32));
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

type S = Wrap<Serial>;

#[doc(hidden)]
pub(crate) static mut LOGGER: Option<Logger> = None;

// type `Serial` is declared outside this crate, avoid orphan rule
pub(crate) struct Wrap<T>(T);

#[doc(hidden)]
pub(crate) struct Logger {
    pub(crate) inner: S,
}

impl fmt::Write for S {
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

pub fn read() -> u8 {
    unsafe {
        match &mut LOGGER {
            Some(l) => {
                if l.inner.0.uart.statr.read().rxne().bit_is_set() {
                    let c = l.inner.0.uart.datar.read().bits() as u8;
                    // l.inner.0.uart.datar.write(|w| w.bits(0 as u32));
                    l.inner.0.uart.statr.write(|w| w.rxne().clear_bit());
                    return c;
                }
                0
            },
            _ => 0
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
