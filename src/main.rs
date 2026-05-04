//! nanoCH32H417 LED Blink
//!
//! Bare-metal Rust for the CH32H417 (QingKe V5F core).
//! Toggles LED1 (PC2) every 500ms using SysTick1 polling delay.
//!
//! # Architecture
//! - `pac.rs` — peripheral register definitions (GPIO, RCC, SysTick)
//! - `main.rs` — startup + blink loop
//!
//! Delay uses SysTick1 polling (same pattern as C SDK Delay_Ms):
//! no interrupts, just busy-wait on SysTick0->ISR bit1.
//!
//! # Hardware
//! - Board: nanoCH32H417
//! - LED1: PC2, LED2: PC3, active-high
//! - Debugger: WCH-LinkE (RV mode)
//!
//! # Flash
//! ```bash
//! cargo run --release
//! ```

#![no_std]
#![no_main]

use core::arch::global_asm;
use panic_halt as _;

mod pac;

use pac::{delay_ms, rcc_enable_gpioc, GpioPin, Pin, GPIOC_BASE};

// ── Startup ────────────────────────────────────────────────────

global_asm!(r#"
.section .init, "ax"
.globl _start
_start:
    la   sp, _stack_start
    la   t0, _sbss
    la   t1, _ebss
1:  beq  t0, t1, 2f
    sw   zero, 0(t0)
    addi t0, t0, 4
    j    1b
2:  jal  zero, rust_main
"#);

// ── Main ───────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // Enable GPIOC peripheral clock
    rcc_enable_gpioc();

    // Configure LED pins
    let led1 = GpioPin::new(GPIOC_BASE, Pin::Pc2);
    led1.init_output();
    let led2 = GpioPin::new(GPIOC_BASE, Pin::Pc3);
    led2.init_output();

    loop {
        // LED1 on, LED2 off
        led1.set_high();
        led2.set_low();
        delay_ms(500);

        // LED1 off, LED2 on
        led1.set_low();
        led2.set_high();
        delay_ms(500);
    }
}
