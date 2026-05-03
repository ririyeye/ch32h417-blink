MEMORY
{
    FLASH  (rx) : ORIGIN = 0x08000000, LENGTH = 480K
    RAM    (rw) : ORIGIN = 0x200A0000, LENGTH = 128K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);

SECTIONS
{
    .text :
    {
        *(.init)
        *(.text .text.*)
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } > FLASH

    .data : AT(LOADADDR(.text) + SIZEOF(.text))
    {
        _sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        . = ALIGN(4);
        _edata = .;
    } > RAM

    .bss (NOLOAD) :
    {
        _sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4);
        _ebss = .;
    } > RAM
}
