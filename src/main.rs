//! nanoCH32H417 LED Blink (SysTick interrupt version)
//!
//! Bare-metal Rust for the CH32H417 (QingKe V5F core).
//! Toggles LED1 (PC2) every 500ms using SysTick1 interrupt + WFI.
//!
//! # Architecture
//! - `pac.rs` — peripheral register definitions + SysTick interrupt handler
//! - `main.rs` — startup (vector table, CSR init) + blink loop
//!
//! # Interrupt flow
//! ```
//! SysTick1 fires → HW pushes regs → mtvec[13] → SysTick1_Handler (asm shim)
//!   → __rust_systick1_handler() clears flag, sets TICK_EXPIRED
//!   → mret (HW pops regs)
//! main loop: systick_delay_ms() → sets up timer → WFI → wakes on TICK_EXPIRED
//! ```
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

use pac::{rcc_enable_gpioc, systick_delay_ms, systick_init, GpioPin, Pin, GPIOC_BASE};

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

2:  // ── Interrupt system config ────────────────────────────────
    // CSR 0xBC0: Prefetch + pipeline control (V5F)
    li   t0, 0x1237B3E0
    csrw 0xBC0, t0
    // CSR 0xBC1: Nesting depth = 8
    li   t0, 0x07
    csrw 0xBC1, t0
    // CSR 0x804: Hardware stacking + interrupt nesting (5-8 level)
    li   t0, 0x0F
    csrw 0x804, t0
    // mstatus: FPU=on, MPP=M, MPIE=1, MIE=1
    li   t0, 0x6088
    csrw mstatus, t0
    // CSR 0x800 (GINTENR): enable global interrupts
    li   t0, 0x88
    csrs 0x800, t0
    // mtvec = _vector_base | 3 (VectoredAddress mode)
    la   t0, _vector_base
    ori  t0, t0, 3
    csrw mtvec, t0

    jal  zero, rust_main
"#);

// ── Vector table ────────────────────────────────────────────────

global_asm!(r#"
.section .vector, "ax"
.align 2
.globl _vector_base
_vector_base:
    // Core exceptions (0-15)
    .word _start               // 0
    .word 0                    // 1
    .word default_handler      // 2  NMI
    .word default_handler      // 3  HardFault
    .word 0                    // 4
    .word default_handler      // 5  Ecall M-Mode
    .word 0                    // 6
    .word 0                    // 7
    .word default_handler      // 8  Ecall U-Mode
    .word default_handler      // 9  Breakpoint
    .word 0                    // 10
    .word 0                    // 11
    .word default_handler      // 12 SysTick0
    .word SysTick1_Handler     // 13 SysTick1 — our timer
    .word default_handler      // 14 SW
    .word 0                    // 15
    // IPC + reserved (16-31)
    .word default_handler      // 16 IPC_CH0
    .word default_handler      // 17 IPC_CH1
    .word default_handler      // 18 IPC_CH2
    .word default_handler      // 19 IPC_CH3
    .word 0; .word 0; .word 0; .word 0; .word 0
    .word 0; .word 0
    .word default_handler      // 28 HSEM
    .word 0; .word 0; .word 0
"#);

// ── Trap handlers (WCH-Interrupt-fast ABI shims) ───────────────

global_asm!(r#"
.section .trap, "ax"
.align 2

.globl SysTick1_Handler
SysTick1_Handler:
    addi sp, sp, -4
    sw   ra, 0(sp)
    jal  __rust_systick1_handler
    lw   ra, 0(sp)
    addi sp, sp, 4
    mret

.globl default_handler
default_handler:
1:  j    1b
"#);

// ── Main ───────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // Enable SysTick1 interrupt in PFIC
    systick_init();

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
        systick_delay_ms(500);

        // LED1 off, LED2 on
        led1.set_low();
        led2.set_high();
        systick_delay_ms(500);
    }
}
