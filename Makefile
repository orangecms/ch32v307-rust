### https://nc-pin.com/index.php/2022/04/25/openocd-for-ch32v-series/ ###
OPENOCD_CFG := ./wch-riscv.cfg
#OPENOCD := /home/dama/firmware/WCH/riscv-openocd-wch/src/openocd
OPENOCD := /home/dama/firmware/WCH/MRS_Toolchain_Linux_x64_V1.60/OpenOCD/bin/openocd
OPENOCD_CMD := $(OPENOCD) -f $(OPENOCD_CFG) -c init
TARGET := target/riscv32imac-unknown-none-elf/release/ch32v307-test
FEATURES := ''

all: build flash

build:
	cargo build --release --features $(FEATURES)

# Program
flash: erase $(TARGET)
	$(OPENOCD_CMD) -c halt -c "program $(TARGET)" -c exit
	#$(OPENOCD_CMD) -c halt -c "verify_image $(TARGET)" -c exit
	#$(OPENOCD_CMD) -c "reset" -c exit || exit 0

erase:
	$(OPENOCD_CMD) -c halt -c "flash erase_sector wch_riscv 0 last" -c exit
