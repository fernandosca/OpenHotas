MEMORY {
    /* RP2350 boots directly from an image definition in flash. Unlike the
     * RP2040, it does not use the 256-byte BOOT2 reservation. */
    FLASH : ORIGIN = 0x10000000, LENGTH = 2048K
    RAM   : ORIGIN = 0x20000000, LENGTH = 520K
}

/* The RP2350 Boot ROM scans the first 4 KiB for this image definition. */
SECTIONS {
    .start_block : ALIGN(4)
    {
        __start_block_addr = .;
        KEEP(*(.start_block));
        KEEP(*(.boot_info));
    } > FLASH
} INSERT AFTER .vector_table;

/* Place executable code after the RP2350 image definition. */
_stext = ADDR(.start_block) + SIZEOF(.start_block);

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
