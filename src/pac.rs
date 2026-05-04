//! CH32H417 Peripheral Access Crate (minimal)
//!
//! Manual register definitions extracted from the C SDK headers:
//!   ch32h417.h, core_riscv.h

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering};

// ── Clocks ─────────────────────────────────────────────────────

pub const HSI_VALUE: u32 = 25_000_000; // 25 MHz internal oscillator

// ── RCC ────────────────────────────────────────────────────────

pub const RCC_BASE: u32 = 0x40021000;
const RCC_HB2PCENR: u32 = 0x1C;

pub fn rcc_enable_gpioc() {
    let addr = (RCC_BASE + RCC_HB2PCENR) as *mut u32;
    unsafe {
        write_volatile(addr, read_volatile(addr) | 0x10); // bit 4 = GPIOC
    }
}

// ── GPIO ───────────────────────────────────────────────────────

const GPIO_BSHR_OFFSET: u32 = 0x10;
const GPIO_CFGLR_OFFSET: u32 = 0x00;
const GPIO_SPEED_OFFSET: u32 = 0x1C;

pub const GPIOC_BASE: u32 = 0x40011000;

#[repr(u32)]
#[allow(unused)]
pub enum Pin {
    Pc2 = 2,
    Pc3 = 3,
}

pub struct GpioPin {
    port_base: u32,
    pin: u32,
}

impl GpioPin {
    pub fn new(port_base: u32, pin: Pin) -> Self {
        Self {
            port_base,
            pin: pin as u32,
        }
    }

    /// Configure pin as push-pull output, max speed
    pub fn init_output(&self) {
        unsafe {
            // CFGLR: 4 bits per pin, CNF[1:0]:MODE[1:0]
            // CNF=00 (push-pull), MODE=01 (output max 50MHz)
            let cfglr = (self.port_base + GPIO_CFGLR_OFFSET) as *mut u32;
            let shift = self.pin * 4;
            let v = read_volatile(cfglr) & !(0xF << shift);
            write_volatile(cfglr, v | (0x1 << shift));

            // SPEED: 2 bits per pin, 0b11 = very high speed
            let speed = (self.port_base + GPIO_SPEED_OFFSET) as *mut u32;
            let s_shift = self.pin * 2;
            let sv = read_volatile(speed) & !(0x3 << s_shift);
            write_volatile(speed, sv | (0x3 << s_shift));
        }
    }

    pub fn set_high(&self) {
        unsafe {
            write_volatile(
                (self.port_base + GPIO_BSHR_OFFSET) as *mut u32,
                1 << self.pin,
            );
        }
    }

    pub fn set_low(&self) {
        unsafe {
            write_volatile(
                (self.port_base + GPIO_BSHR_OFFSET) as *mut u32,
                1 << (self.pin + 16),
            );
        }
    }
}

// ── SysTick ─────────────────────────────────────────────────────

pub const STK_CTLR_OFFSET: u32 = 0x00;
pub const STK_ISR_OFFSET: u32 = 0x04;
pub const STK_CNT_OFFSET: u32 = 0x08;
pub const STK_CMP_OFFSET: u32 = 0x10;

/// SysTick1 base (V5F uses SysTick1 for timing)
pub const STK1_BASE: u32 = 0xE000F080;
/// SysTick0 base (used for ISR flags of both timers)
pub const STK0_BASE: u32 = 0xE000F000;


/// Get HCLK frequency. After reset, MCU runs on HSI.
pub fn hclk() -> u32 {
    HSI_VALUE
}

// ── PFIC ───────────────────────────────────────────────────────

/// PFIC Interrupt Enable Register 0 (IRQ 0-31)
const PFIC_IENR0: *mut u32 = 0xE000E100 as *mut u32;
/// PFIC Interrupt Priority base (8-bit per IRQ)
const PFIC_IPRIOR: *mut u8 = 0xE000E400 as *mut u8;
/// PFIC Interrupt Allocate (IALLOCR): 0=V3F, 1=V5F
const PFIC_IALLOCR: *mut u8 = 0xE000E500 as *mut u8;

/// Initialize SysTick1 interrupt in PFIC.
/// Must call once before using systick_delay_ms().
pub fn systick_init() {
    unsafe {
        // Route SysTick1 (IRQ 13) to V5F core.
        // Reset default is 0 (V3F), but V3F is sleeping → interrupt lost.
        write_volatile(PFIC_IALLOCR.offset(13), 1u8);

        // Set SysTick1 (IRQ 13) priority = 0 (lowest)
        write_volatile(PFIC_IPRIOR.offset(13), 0u8);
        // Enable SysTick1 in PFIC IENR0 (bit 13)
        let ienr = read_volatile(PFIC_IENR0);
        write_volatile(PFIC_IENR0, ienr | (1 << 13));
    }
}

/// Interrupt flag: set by SysTick1 handler, checked by delay loop
pub static TICK_EXPIRED: AtomicBool = AtomicBool::new(false);

/// One-shot interrupt-driven delay using SysTick1.
/// Puts the core into WFI sleep while waiting.
pub fn systick_delay_ms(ms: u32) {
    let ticks = hclk() / 1000 * ms;

    TICK_EXPIRED.store(false, Ordering::Release);

    unsafe {
        // Clear SysTick1 flag (bit 1 in SysTick0.ISR, write-0-to-clear)
        let stk0_isr = (STK0_BASE + STK_ISR_OFFSET) as *mut u32;
        write_volatile(stk0_isr, read_volatile(stk0_isr) & !(1 << 1));

        // Reset counter, set compare
        write_volatile((STK1_BASE + STK_CNT_OFFSET) as *mut u32, 0);
        write_volatile((STK1_BASE + STK_CMP_OFFSET) as *mut u32, ticks);

        // Start: bit3=HCLK, bit2=clear-cnt, bit1=interrupt-en, bit0=counter-en
        write_volatile(
            (STK1_BASE + STK_CTLR_OFFSET) as *mut u32,
            (1 << 3) | (1 << 2) | (1 << 1) | (1 << 0),
        );
    }

    // Wait for interrupt handler to signal completion
    while !TICK_EXPIRED.load(Ordering::Acquire) {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

/// SysTick1 interrupt handler (called from assembly shim).
/// Clears the interrupt flag and sets TICK_EXPIRED.
#[unsafe(no_mangle)]
extern "C" fn __rust_systick1_handler() {
    unsafe {
        // Read SysTick1 flag from SysTick0.ISR bit1
        let isr = read_volatile((STK0_BASE + STK_ISR_OFFSET) as *const u32);
        if isr & (1 << 1) != 0 {
            // Write-0-to-clear: SysTick0->ISR &= ~(1<<1)
            write_volatile(
                (STK0_BASE + STK_ISR_OFFSET) as *mut u32,
                isr & !(1 << 1),
            );

            // Stop timer
            write_volatile((STK1_BASE + STK_CTLR_OFFSET) as *mut u32, 0);

            // Signal main loop
            TICK_EXPIRED.store(true, Ordering::Release);
        }
    }
}
