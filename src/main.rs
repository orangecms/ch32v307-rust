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

fn sleep(t: i32) {
    for _ in 0..t {
        unsafe {
            riscv::asm::nop();
        }
    }
}

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
            .cfglr
            .modify(|_, w| w.cnf5().bits(0b00).mode5().bits(0b11).cnf6().bits(0b00).mode6().bits(0b11).cnf7().bits(0b00).mode7().bits(0b11));
        gpiob
            .cfghr
            .modify(|_, w| w.cnf8().bits(0b00).mode8().bits(0b11).cnf9().bits(0b00).mode9().bits(0b11));
    };

    // println!("Hello, world!");
    // HSI 8MHz
    // 4 opcodes to do a nop sleep here
    let cycle = 8_000_000 / 4;
    loop {
        gpiob.outdr.modify(|_, w| w.odr5().set_bit());
        gpiob.outdr.modify(|_, w| w.odr7().set_bit());
        sleep(cycle);
        gpiob.outdr.modify(|_, w| w.odr6().set_bit());
        gpiob.outdr.modify(|_, w| w.odr8().set_bit());
        sleep(cycle);
        gpiob.outdr.modify(|_, w| w.odr9().set_bit());
        sleep(cycle);

        gpiob.outdr.modify(|_, w| w.odr5().clear_bit());
        gpiob.outdr.modify(|_, w| w.odr7().clear_bit());
        sleep(cycle);
        gpiob.outdr.modify(|_, w| w.odr6().clear_bit());
        gpiob.outdr.modify(|_, w| w.odr8().clear_bit());
        sleep(cycle);
        gpiob.outdr.modify(|_, w| w.odr9().clear_bit());
        sleep(cycle);
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
