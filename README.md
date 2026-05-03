# ch32h417-blink

Bare-metal GPIO blink for **CH32H417** (QingKe-V5F RISC-V core) in Rust.

No HAL, no PAC, no runtime — just `global_asm` startup + direct register writes.

## Quick Start

```bash
rustup target add riscv32imac-unknown-none-elf
git clone https://github.com/ririyeye/ch32h417-blink
cd ch32h417-blink
cargo run --release
```

`cargo run` builds the ELF and flashes via [probe-rs](https://github.com/ririyeye/probe-rs)
(at `../probe-rs/target/release/probe-rs` relative to the project).

## How It Works

```
_start (global_asm)
  ├── la sp, _stack_start        # set stack pointer
  └── jal rust_main              # jump to Rust

rust_main()
  ├── RCC->HB2PCENR |= 0x10      # enable GPIOC clock
  ├── GPIOC->CFGLR  = output     # PC2, PC3 = push-pull
  ├── GPIOC->SPEED  = max        # max speed
  └── loop:
        ├── PC2=HI, PC3=LO, delay ~500ms (HSI 8 MHz)
        └── PC2=LO, PC3=HI, delay ~500ms
```

## Hardware

- **Chip**: CH32H417 (QingKe-V5F RISC-V core)
- **Board**: nanoCH32H417
- **Debugger**: WCH-LinkE (RV mode)
- **LEDs**: PC2 (LED1), PC3 (LED2), active-high
- **Clock**: HSI 8 MHz internal oscillator

## Dependencies

| Crate | Why |
|-------|-----|
| `panic-halt` | Halt on panic |
| *(none else)* | All registers via `write_volatile` / `read_volatile` |

Binary size: ~100 bytes.

## Related

- [ch32h417-async](https://github.com/ririyeye/ch32h417-async) — async version with custom runtime
- [probe-rs CH32H417](https://github.com/ririyeye/probe-rs) — flash tooling
- [nanoCH32H417](https://github.com/wuxx/nanoCH32H417) — official dev board SDK

## License

MIT
