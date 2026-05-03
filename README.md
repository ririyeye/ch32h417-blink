# ch32h417-blink

Rust embedded GPIO blink example for CH32H417 (QingKe-V5F core).

Flashes PC2 and PC3 alternately (on-board LEDs on nanoCH32H417 dev board).

## Build

```bash
rustup target add riscv32imac-unknown-none-elf
cargo build --release
```

## Flash

```bash
probe-rs download --chip CH32H417 --chip-erase \
  --binary-format elf \
  target/riscv32imac-unknown-none-elf/release/ch32h417-blink
```

## Hardware

- Target: CH32H417 (QingKe-V5F RISC-V core)
- Debugger: WCH-LinkE (RV mode)
- LED1: PC2, LED2: PC3 (nanoCH32H417 board)
- Clock: HSE 25MHz → PLL → 144MHz system clock
