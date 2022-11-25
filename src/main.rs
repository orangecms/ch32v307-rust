#![no_std]
#![no_main]

use riscv_rt::entry;
use core::{
    arch::asm,
    panic::PanicInfo,
    ptr::{slice_from_raw_parts, write_volatile},
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
    rcc.ctlr.modify(|_, w| w.pllon().set_bit().pll2on().set_bit().pll3on().set_bit());
    rcc.ctlr.modify(|_, w| w.csson().set_bit().hseon().clear_bit());
    rcc.ctlr.modify(|_, w| w.hseon().set_bit());
    while rcc.ctlr.read().hserdy() != true {}

    rcc.ctlr.modify(|_, w| w.pllon().set_bit());
    while rcc.ctlr.read().pllrdy() != true {}

    unsafe {
        rcc.cfgr0.modify(|_, w| w.ppre1().bits(0b100));
        rcc.cfgr0.modify(|_, w| w.pllsrc().set_bit().pllxtpre().set_bit());
        // PLLMUL 0b0111 means 9x 8MHz = 72MHz
        rcc.cfgr0.modify(|_, w| w.mco().bits(0b1000).pllmul().bits(0b0111));
        // use PLL oscillator as system clock
        rcc.cfgr0.modify(|_, w| w.sw().bits(0b10));
    }

    // enable IO ports A and B, as well as UART1
    rcc.apb2pcenr.modify(|_, w| w.iopaen().set_bit().iopben().set_bit().usart1en().set_bit());

    // configure GPIOs
    let gpioa = &peripherals.GPIOA;
    let gpiob = &peripherals.GPIOB;

    unsafe {
        gpioa.cfglr.modify(|_, w| w.cnf0().bits(0b00).mode0().bits(0b11));
        gpioa
            .cfghr
            .modify(|_, w| w.cnf9().bits(0b10).mode9().bits(0b11).cnf10().bits(0b10).mode10().bits(0b11));
        gpiob
            .cfglr
            .modify(|_, w| w.cnf5().bits(0b00).mode5().bits(0b11).cnf6().bits(0b00).mode6().bits(0b11).cnf7().bits(0b00).mode7().bits(0b11));
        gpiob
            .cfghr
            .modify(|_, w| w.cnf8().bits(0b00).mode8().bits(0b11).cnf9().bits(0b00).mode9().bits(0b11));
    };

    gpioa.outdr.modify(|_, w| w.odr0().set_bit());

    let uart1 = &peripherals.USART1;

    unsafe {
        // 12 bits mantissa, last 4 bits are fraction (1/16)
        uart1.ctlr1.modify(|_, w| w.ue().set_bit().m().clear_bit());
        uart1.ctlr1.modify(|_, w| w.te().set_bit());
        uart1.ctlr2.modify(|_, w| w.stop().bits(0b00));
        uart1.brr.modify(|_, w| w.div_mantissa().bits(39).div_fraction().bits(1));

        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits('w' as u32));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits('o' as u32));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits('o' as u32));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits('p' as u32));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits(' ' as u32));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits('w' as u32));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits('o' as u32));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits('o' as u32));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits('p' as u32));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits(' ' as u32));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits(0xf0));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits(0x9f));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits(0xa6));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits(0x80));
        while uart1.statr.read().txe() != true {}
        uart1.datar.modify(|_, w| w.bits('\n' as u32));
    }

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
