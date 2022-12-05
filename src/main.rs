#![no_std]
#![no_main]

use core::arch::asm;
use riscv_rt::entry;
use panic_halt as _;
// riscv provides implementation for critical-section
use riscv::{self as _, register::mstatus::Mstatus};

use ch32v3::ch32v30x;

#[macro_use]
mod log;

#[export_name = "MachineSoft"]
fn soft_handler() {
    println!("MachineSoft");
}

#[export_name = "MachineExternal"]
fn ext_handler() {
    println!("MachineExternal");
}

#[export_name = "UserSoft"]
fn usoft_handler() {
    println!("UserSoft");
}

#[cfg(feature = "int")]
// custom default handler
#[export_name = "DefaultHandler"]
fn default_handler(irqn: i16) {
    // systick timer
    if irqn == 12 {
        unsafe {
            // println!("{:x}", riscv::register::mip::read().bits());
            XX = !XX;
            // stay in machine mode after interrupt handling, see also
            // https://github.com/rust-embedded/riscv-rt/pull/42/files
            riscv::register::mstatus::set_mpp(
                riscv::register::mstatus::MPP::Machine
            );
            asm!("mret");
        }
        // TODO: How do we clear the interrupts?!
        // return;
    }
    // usart1
    if irqn == 53 {
        echo();
        return;
    }
    let mstatus = riscv::register::mstatus::read();
    let mcause = riscv::register::mcause::read();
    let mtval = riscv::register::mtval::read();
    let cause = mcause.cause();
    let code = mcause.code();
    let itype = if mcause.is_interrupt() { "interrupt" } else { "exception" };
    println!("Interrupt IRQ {}, status {:?} tval {:x}", irqn, mstatus, mtval);
    println!("Interrupt code {} cause {:?} type {itype}", code, cause);
    where_am_i();
}

#[no_mangle]
fn stk_handler() {
    print!("stk");
    unsafe {
        XX = true;
    }
}

fn sleep(t: i32) {
    for _ in 0..t {
        unsafe {
            riscv::asm::nop();
        }
    }
}

static mut XX: bool = false;

#[no_mangle]
fn echo() {
    print!("echo");
    unsafe {
        XX = true;
    }
}

const RISCV_BANNER: &str = r"
 ____   ___  ____    ____     __     __
|  _ \ |_ _|/ ___|  / ___|    \ \   / /
| |_) | | | \___ \ | |    _____\ \ / /
|  _ <  | |  ___) || |___|_____|\ V /
|_| \_\|___||____/  \____|       \_/
";

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
    rcc.ctlr.modify(|_, w| w.pllon().set_bit().pll2on().set_bit());
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
    rcc.apb2pcenr.modify(|_, w| w.iopaen().set_bit()
                                 .iopben().set_bit()
                                 .usart1en().set_bit());

    // configure GPIOs
    let gpioa = &peripherals.GPIOA;
    let gpiob = &peripherals.GPIOB;
    let afio = &peripherals.AFIO;

    unsafe {
        gpioa.cfglr.modify(|_, w| w.cnf0().bits(0b00).mode0().bits(0b11));
        // confire A9/A10 for UART TX/RX
        gpioa
            .cfghr
            .modify(|_, w| w.cnf9().bits(0b10).mode9().bits(0b11)
                            .cnf10().bits(0b10).mode10().bits(0b00));
        gpiob
            .cfglr
            .modify(|_, w| w.cnf5().bits(0b00).mode5().bits(0b11)
                            .cnf6().bits(0b00).mode6().bits(0b11)
                            .cnf7().bits(0b00).mode7().bits(0b11));
        gpiob
            .cfghr
            .modify(|_, w| w.cnf8().bits(0b00).mode8().bits(0b11)
                            .cnf9().bits(0b00).mode9().bits(0b11)
            );

        // enable event output
        afio.ecr.modify(|_, w| w.evoe().set_bit());
        afio.exticr3.modify(|_, w| w.exti10().bits(0000));
    };

    gpioa.outdr.modify(|_, w| w.odr0().set_bit());

    let serial = log::Serial::new(peripherals.USART1);
    log::set_logger(serial);
    ch32v3::interrupt!(USART1, echo);
    ch32v3::interrupt!(STK, stk_handler);

    /*
     * Steps to use external hardware interrupt:
     * 1) Configure GPIO;
     * 2) Configure the EXTI_INTENR bit in the corresponding external interrupt
     *    channel;
     * 3) Configure the trigger edge (EXTI_RTENR or EXTI_FTENR), select rising
     * edge trigger, falling edge trigger or double edges trigger;
     * 4) Configure the EXTI interrupt in the NVIC/PFIC of the core to ensure 
     * that it can respond correctly.
     *
     * Steps to use external hardware event:
     * 1) Configure GPIO;
     * 2) Configure the EXTI_EVENR bit in the corresponding external interrupt
     * channel;
     * 3) Configure the trigger edge (EXTI_RTENR or EXTI_FTENR), select rising
     * edge trigger, falling edge trigger or double edges trigger.
     *
     * Steps to use software interrupt/event:
     * 1) Enable external interrupt (EXTI_INTENR) or external event
     * (EXTI_EVENR);
     * 2) To use the interrupt service function, set the EXTI interrupt in the
     * NVIC/PEIC of the core;
     * 3) Set the software interrupt trigger (EXTI_SWIEVR) to generate an
     * interrupt.
     */

    unsafe {
        let exti = peripherals.EXTI;
        exti.intenr.modify(|_, w| w.bits(0xffff));
        exti.evenr.modify(|_, w| w.bits(0xffff));
        exti.rtenr.write(|w| w.bits(0xffff));
        exti.ftenr.write(|w| w.bits(0xffff));
        exti.swievr.write(|w| w.bits(0xffff));

        let pfic = peripherals.PFIC;
        pfic.ienr1.write(|w| w.bits(0xffff));
        // triggers when RISC-V interrupts enabled
        // - interrupt 39 aka EXTI9_5 (EXTI Line\[9:5\])
        // pfic.ienr2.write(|w| w.bits(0x0080));
        // interrupt 37, USART1
        pfic.ienr2.write(|w| w.bits((37 - 32) << 1));
        // pfic.ienr2.write(|w| w.bits(0xffff));
        pfic.ienr3.write(|w| w.bits(0xffff));
        pfic.ienr4.write(|w| w.bits(0xffff));
    }

    // count count
    let stk = &peripherals.SYSTICK;
    stk.ctlr.modify(|_, w| w.stre().set_bit() // auto reset
                            .stie().set_bit() // enable interrupt
                            .ste().set_bit() // enable counter
    );
    unsafe {
        stk.cmphr.write(|w| w.bits(0x0000_0000));
        stk.cmplr.write(|w| w.bits(0x0123_abcd));
    }

    println!("{RISCV_BANNER}");
    println!("The meaning of life is to rewrite everything in Rust. 🦀🦀");

    machine_info();
    where_am_i();
    unsafe {
        riscv::interrupt::enable();
        riscv::register::mie::set_usoft();
        riscv::register::mip::set_usoft();
    }
    println!("Type something!");
    let mut inp: u8 = 0;
    loop {
        let v = stk.cntl.read().bits();
        if v > 0x0100_0100 {
            gpiob.outdr.modify(|_, w| w.odr9().set_bit());
        } else {
            gpiob.outdr.modify(|_, w| w.odr9().clear_bit());
        }
        /*
        if (v >> 2) % 0x0008_0000 == 0 {
            println!("it's the final ... {}", v);
        }
        */
        // FIXME: we want interrupts...
        let x = log::read();
        if x != 0 {
            match (x as char, x) {
                ('\r', _) => {
                    if inp as char == 'r' {
                        println!(" 🦀");
                    } else {
                        println!(" 🐢");
                    }
                },
                (_x, 0x08) => {
                    print!("{_x}{_x}🩹");
                },
                ('w', _) => {
                    print!("🧇");
                },
                (_x, _) => {
                    print!("{}", _x);
                },
            }
            inp = x;
        }
        unsafe {
            if XX {
                where_am_i();
                XX = false;
            }
        }
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
