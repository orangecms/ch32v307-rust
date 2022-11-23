#![no_std]
#![no_main]

use riscv_rt::entry;
use core::{
    arch::asm,
    panic::PanicInfo,
    ptr::slice_from_raw_parts,
};
// use embedded_hal::serial::nb::Write;
use panic_halt as _;
// riscv provides implementation for critical-section
use riscv as _;

use ch32v3::ch32v30x;

#[no_mangle]
extern "C" fn DefaultHandler() {}

#[entry]
fn main() -> ! {
    let peripherals = ch32v30x::Peripherals::take().unwrap();

    let rcc = peripherals.RCC;
    rcc.apb2pcenr.modify(|_, w| w.iopben().set_bit());

    let gpioa = &peripherals.GPIOA;
    gpioa.outdr.modify(|_, w| w.odr0().set_bit());

    let gpiob = &peripherals.GPIOB;

    // Output max 50MHz
    // Push-pull
    unsafe {
        gpiob
            .cfghr
            .modify(|_, w| w.cnf8().bits(0b00).mode8().bits(0b11))
    };

    // println!("Hello, world!");
    // HSI 8MHz
    // 4 opcodes to do a nop sleep here
    let cycle = 8_000_000 / 4;
    loop {
        gpiob.outdr.modify(|_, w| w.odr8().set_bit());
        for _ in 0..cycle {
            unsafe {
                riscv::asm::nop();
            }
        }

        gpiob.outdr.modify(|_, w| w.odr8().clear_bit());
        for _ in 0..cycle {
            unsafe {
                riscv::asm::nop();
            }
        }
    }
}
/*
#[cfg_attr(not(test), panic_handler)]
fn panic(info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
*/
