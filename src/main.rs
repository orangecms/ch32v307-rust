#![no_std]
#![no_main]

use riscv_rt::entry;
use core::{
    arch::asm,
    panic::PanicInfo,
    ptr::{slice_from_raw_parts, write_volatile},
};
use panic_halt as _;
// riscv provides implementation for critical-section
use riscv as _;

use ch32v3::{ch32v30x, ch32v30x::Interrupt::USART1};

#[macro_use]
mod log;

#[export_name = "MachineSoft"]
fn soft_handler() {
    print!("x");
}

#[export_name = "MachineExternal"]
fn ext_handler() {
    print!("x");
}

#[export_name = "UserSoft"]
fn usoft_handler() {
    print!("x");
}

#[export_name = "DefaultHandler"]
fn default_handler(irqn: i16) {
    // custom default handler
    print!("x");
}
/*
#[no_mangle]
extern "C" fn DefaultHandler() {}
*/

fn sleep(t: i32) {
    for _ in 0..t {
        unsafe {
            riscv::asm::nop();
        }
    }
}

fn echo() {
    print!("x");
}

/* see https://five-embeddev.com/riscv-isa-manual/latest/machine.html */
fn machine_info() {
    match riscv::register::misa::read() {
        None => { println!("ISA unknown"); },
        Some(v) => { println!("ISA: {:?}", v); },
    }

    match riscv::register::mvendorid::read() {
        None => { println!("vendor unknown"); },
        Some(v) => { println!("vendor: {:?}", v); },
    }

    match riscv::register::mimpid::read() {
        None => { println!("impl. ID unknown"); },
        Some(v) => { println!("impl. ID: {:?}", v); },
    }
}

fn where_am_i() {
    let mpc = riscv::register::mepc::read();
    println!("Where am I? {}", mpc);
    // let spc = riscv::register::sepc::read();
    // println!("Where is she? {:x}", spc);
    // let upc = riscv::register::uepc::read();
    // println!("Where are you? {}", upc);
}

// HSI 8MHz
// 4 opcodes to do a nop sleep here
const CYCLE: i32 = 8_000_000 / 4;

fn blink(gpiob: &ch32v3::ch32v30x::GPIOB) {
    gpiob.outdr.modify(|_, w| w.odr5().set_bit());
    gpiob.outdr.modify(|_, w| w.odr7().set_bit());
    sleep(CYCLE);
    gpiob.outdr.modify(|_, w| w.odr6().set_bit());
    gpiob.outdr.modify(|_, w| w.odr8().set_bit());
    sleep(CYCLE);
    gpiob.outdr.modify(|_, w| w.odr9().set_bit());
    sleep(CYCLE);

    gpiob.outdr.modify(|_, w| w.odr5().clear_bit());
    gpiob.outdr.modify(|_, w| w.odr7().clear_bit());
    sleep(CYCLE);
    gpiob.outdr.modify(|_, w| w.odr6().clear_bit());
    gpiob.outdr.modify(|_, w| w.odr8().clear_bit());
    sleep(CYCLE);
    gpiob.outdr.modify(|_, w| w.odr9().clear_bit());
    sleep(CYCLE);
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
            .modify(|_, w| w.cnf9().bits(0b10).mode9().bits(0b11).cnf10().bits(0b10).mode10().bits(0b00));
        gpiob
            .cfglr
            .modify(|_, w| w.cnf5().bits(0b00).mode5().bits(0b11).cnf6().bits(0b00).mode6().bits(0b11).cnf7().bits(0b00).mode7().bits(0b11));
        gpiob
            .cfghr
            .modify(|_, w| w.cnf8().bits(0b00).mode8().bits(0b11).cnf9().bits(0b00).mode9().bits(0b11));
    };

    gpioa.outdr.modify(|_, w| w.odr0().set_bit());

    ch32v3::interrupt!(USART1, echo);

    let serial = log::Serial::new(peripherals.USART1);
    log::set_logger(serial);
    println!("The meaning of life is to rewrite everything in Rust. 🦀🦀");

    machine_info();
    where_am_i();

    unsafe {
        riscv::interrupt::enable();
        riscv::register::mie::set_usoft();
        riscv::register::mip::set_usoft();
    }

    println!("Type something!");
    use core::fmt;
    use fmt::Write;
    loop {
        let x = log::read();
        if x != 0 {
            print!("{}", x as char);
            if x as char == '\r' {
                println!();
            }
        }
        // where_am_i();
        // blink(gpiob);
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
