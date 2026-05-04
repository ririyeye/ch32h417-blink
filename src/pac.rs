//! CH32H417 Peripheral Access Crate (minimal)
//!
//! Manual register definitions extracted from the C SDK headers:
//!   ch32h417.h, core_riscv.h

use core::ptr::{read_volatile, write_volatile};

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
/// If you configure PLL, update this accordingly.
pub fn hclk() -> u32 {
    HSI_VALUE
}

/// Polling delay using SysTick1 (V5F timer).
/// Does NOT use interrupts — purely polling.
pub fn delay_ms(ms: u32) {
    let ticks = hclk() / 1000 * ms;

    unsafe {
        // Clear SysTick1 flag (bit 1 in SysTick0.ISR, write-1-to-clear... wait -
        // actually it's write-0-to-clear per C SDK: SysTick0->ISR &= ~(1<<1))
        let stk0_isr = (STK0_BASE + STK_ISR_OFFSET) as *mut u32;
        write_volatile(stk0_isr, read_volatile(stk0_isr) & !(1 << 1));

        // Reset counter
        write_volatile((STK1_BASE + STK_CNT_OFFSET) as *mut u32, 0);
        // Set compare value
        write_volatile((STK1_BASE + STK_CMP_OFFSET) as *mut u32, ticks);
        // Start: bit2=clear-cnt, bit0=enable, clock=HCLK
        write_volatile(
            (STK1_BASE + STK_CTLR_OFFSET) as *mut u32,
            (1 << 2) | (1 << 0),
        );

        // Wait for flag
        while read_volatile(stk0_isr) & (1 << 1) == 0 {}

        // Stop timer
        write_volatile((STK1_BASE + STK_CTLR_OFFSET) as *mut u32, 0);
    }
}
