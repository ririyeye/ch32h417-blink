#![no_std]
#![no_main]

use core::arch::global_asm;
use core::ptr::{read_volatile, write_volatile};
use panic_halt as _;

global_asm!(r#"
.section .init, "ax"
.globl _start
_start:
    la sp, _stack_start
    jal zero, rust_main

.section .trap, "ax"
.globl trap_entry
trap_entry:
    wfi
    j trap_entry
"#);

// ── CH32H417 Peripheral Registers ───────────────────────────

const RCC_HB2PCENR: u32 = 0x40021000 + 0x1C;

const GPIOC_BASE: u32   = 0x40011000;
const GPIOC_CFGLR: u32  = GPIOC_BASE + 0x00;
const GPIOC_BSHR: u32   = GPIOC_BASE + 0x10;
const GPIOC_SPEED: u32  = GPIOC_BASE + 0x1C;

const PC2_SET: u32 = 1 << 2;
const PC2_RST: u32 = 1 << (16 + 2);
const PC3_SET: u32 = 1 << 3;
const PC3_RST: u32 = 1 << (16 + 3);

// ── Delay (HSI=8MHz → ~4000 cycles per ms) ─────────────────

fn delay_ms(ms: u32) {
    for _ in 0..(ms * 4000) {
        unsafe { core::arch::asm!("nop"); }
    }
}

// ── Entry ───────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // Enable GPIOC clock
    unsafe {
        write_volatile(RCC_HB2PCENR as *mut u32,
            read_volatile(RCC_HB2PCENR as *mut u32) | 0x10);
    }

    // Configure PC2 + PC3 as push-pull output.
    // CH32H417 CFGLR per-pin 4 bits: [CNF1:CNF0:MODE1:MODE0]
    // CNF=00 = general purpose push-pull, MODE=01 = output mode
    // Speed via SPEED register: 2 bits per pin, 3 = max
    unsafe {
        let cfglr = GPIOC_CFGLR as *mut u32;
        let v = read_volatile(cfglr) & !(0xFF << 8);
        write_volatile(cfglr, v | (0x1 << 8) | (0x1 << 12));

        let speed = GPIOC_SPEED as *mut u32;
        let sv = read_volatile(speed) & !(0xF << 4);
        write_volatile(speed, sv | (0x3 << 4) | (0x3 << 6));
    }

    loop {
        unsafe { write_volatile(GPIOC_BSHR as *mut u32, PC2_SET | PC3_RST); }
        delay_ms(500);
        unsafe { write_volatile(GPIOC_BSHR as *mut u32, PC2_RST | PC3_SET); }
        delay_ms(500);
    }
}
