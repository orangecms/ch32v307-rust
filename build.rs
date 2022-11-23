use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

// NOTE: We omit 4 bytes here so that the same binary can be used for flashing.
// In flash, the first 4 bytes enode the size of the binary to load into SRAM.
const FLASH: &[u8] = b"
OUTPUT_ARCH(riscv)
ENTRY(_start)
MEMORY {
    FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 288K
    RAM (xrw) : ORIGIN = 0x20000000, LENGTH = 32K
}
SECTIONS {
    .head : {
        *(.head.text)
        KEEP(*(.debug))
        KEEP(*(.bootblock.boot))
    } >FLASH AT>FLASH
    .text : {
        KEEP(*(.text.entry))
        *(.text .text.*)
    } >FLASH AT>FLASH
    .rodata : ALIGN(8) {
        srodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        . = ALIGN(8);
        erodata = .;
    } >RAM AT>FLASH
    .data : ALIGN(8) {
        sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        . = ALIGN(8);
        edata = .;
    } >RAM AT>FLASH
    sidata = LOADADDR(.data);
    .bss (NOLOAD) : ALIGN(4) {
        *(.bss.uninit)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        ebss = .;
    } >RAM AT>FLASH
    /DISCARD/ : {
        *(.eh_frame)
    }
}";

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("link.ld"))
        .unwrap()
        .write_all(FLASH)
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
}
