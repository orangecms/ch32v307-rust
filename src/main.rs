#![feature(naked_functions, asm_sym, asm_const)]
#![no_std]
#![no_main]
use ch32v3::ch32v30x;
use core::{
    arch::asm,
    panic::PanicInfo,
    ptr::slice_from_raw_parts,
};
// use embedded_hal::serial::nb::Write;
use riscv;

const STACK_SIZE: usize = 1 * 1024; // 1KiB

#[link_section = ".bss.uninit"]
static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

/// Set up stack and jump to executable code.
///
/// # Safety
///
/// Naked function.
#[naked]
#[export_name = "start"]
#[link_section = ".text.entry"]
pub unsafe extern "C" fn start() -> ! {
    asm!(
        "0:",
        "li t4, 0x43",
        "li t5, 0x12440000",
        "sw t4, 0(t5)",
        "j 0b", // debug: CCCCCCCCCCC
        // Clear feature disable CSR
        // "csrwi  0x7c1, 0",

        "csrw   mie, zero",
        "csrw   mstatus, zero",
        // 2. initialize programming language runtime
        // clear bss segment
        "la     t0, sbss",
        "la     t1, ebss",
        "1:",
        "bgeu   t0, t1, 1f",
        "sw     x0, 0(t0)",
        "addi   t0, t0, 4",
        "j      1b",
        "1:",
        // 3. prepare stack
        "la     sp, {stack}",
        "li     t0, {stack_size}",
        "add    sp, sp, t0",
        // "j _debug",
        "call   {main}",
        stack      =   sym STACK,
        stack_size = const STACK_SIZE,
        main       =   sym main,
        options(noreturn)
    )
}

fn main() {
    let mut peripherals = unsafe { ch32v30x::Peripherals::steal() };
    let gpioa = &peripherals.GPIOA;
    gpioa.outdr.modify(|_, w| w.odr0().set_bit());

    // println!("Hello, world!");
}

#[cfg_attr(not(test), panic_handler)]
fn panic(info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
