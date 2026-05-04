MEMORY
{
    /* V5F flash: 480KB (of the 960KB total — other half for V3F) */
    FLASH (rx) : ORIGIN = 0x08000000, LENGTH = 480K

    /* V5F ITCM (Instruction Tightly Coupled Memory) */
    RAM  (rw)  : ORIGIN = 0x200A0000, LENGTH = 128K
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
        _sidata = LOADADDR(.data);
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
