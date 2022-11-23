use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const FLASH: &[u8] = b"
OUTPUT_ARCH(riscv)
ENTRY(_start)

__stack_size = 2048;

PROVIDE( _stack_size = __stack_size );

MEMORY {
	FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 288K
	RAM (xrw) : ORIGIN = 0x20000000, LENGTH = 32K
}

SECTIONS {
	/*
	.vector: {
		*(.vector);
		. = ALIGN(64);
	} >FLASH AT>FLASH
	*/
	.head : {
		*(.head.text)
		KEEP(*(.debug))
		KEEP(*(.bootblock.boot))
	} >FLASH AT>FLASH
	.text : {
		. = ALIGN(4);
		KEEP(*(.text.entry))
		*(.text .text.*)
		. = ALIGN(4);
	} >FLASH AT>FLASH 
	.rodata : ALIGN(8) {
		srodata = .;
		*(.rodata .rodata.*)
		*(.srodata .srodata.*)
		. = ALIGN(8);
		erodata = .;
	} >RAM AT>FLASH
	.data : ALIGN(8) {
		PROVIDE( __global_pointer$ = . + 0x800 );
		sdata = .;
		*(.data .data.*)
		*(.sdata .sdata.*)
		. = ALIGN(8);
		edata = .;
	} >RAM AT>FLASH
	sidata = LOADADDR(.data);
	.bss : ALIGN(4) {
		. = ALIGN(4);
		*(.bss.uninit)
		*(.bss .bss.*)
		*(COMMON*)
		. = ALIGN(4);
        ebss = .;
	} >RAM AT>FLASH

	PROVIDE( end = . );
	/DISCARD/ : {
		*(.eh_frame)
		*(.debug_*)
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
