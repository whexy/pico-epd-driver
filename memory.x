/*------------------------------------------------------------------------------
 * RP2350 memory map for Rust (cortex-m-rt + Embassy)
 *
 * Design goals:
 *  - Keep the standard section layout from cortex-m-rt (vectors/.text/.rodata/.data/.bss/.uninit).
 *  - Make the entire contiguous RAM window usable for a large runtime heap.
 *  - Keep a predictable stack at the top of main striped SRAM.
 *  - Retain picotool/boot metadata blocks.
 *
 * Heap/Stack model:
 *  - Stack lives at the top of the main striped RAM region and grows downward.
 *  - Heap starts at `cortex_m_rt::heap_start()` (after .bss/.uninit) and can grow upward
 *    across RAM → SRAM4 → SRAM5 (these three regions are contiguous on RP2350).
 *  - Rust side should compute heap_size = _heap_ceiling - heap_start - safety_margin.
 *----------------------------------------------------------------------------*/

MEMORY
{
  /* External XIP flash mapped at 0x1000_0000. Adjust LENGTH to your board. */
  FLASH  (rx)  : ORIGIN = 0x10000000, LENGTH = 2048K

  /* Striped SRAM banks (SRAM0..SRAM7): high bandwidth, 512 KiB, contiguous. */
  RAM    (rwx) : ORIGIN = 0x20000000, LENGTH = 512K

  /* Direct-mapped scratch banks, contiguous after RAM.
   * Keep them separate so you can later dedicate them (e.g., DMA, per-core stacks)
   * without rewriting the map. For now we fold them into the heap ceiling. */
  SRAM4  (rwx) : ORIGIN = 0x20080000, LENGTH = 4K
  SRAM5  (rwx) : ORIGIN = 0x20081000, LENGTH = 4K
}

/*------------------------------------------------------------------------------
 * Minimal, additive sections
 * We rely on cortex-m-rt's default `link.x` for the core sections.
 * We only insert small metadata blocks required by Pico tooling.
 *----------------------------------------------------------------------------*/

/* Boot ROM info immediately after vector table (kept in the first 4 KiB of flash). */
SECTIONS
{
  .start_block : ALIGN(4)
  {
    __start_block_addr = .;
    KEEP(*(.start_block));
    KEEP(*(.boot_info));
  } > FLASH
} INSERT AFTER .vector_table;

/* Move .text to start after the boot info (keeps the first page tidy). */
_stext = ADDR(.start_block) + SIZEOF(.start_block);

/* Picotool 'Binary Info' table after .text */
SECTIONS
{
  .bi_entries : ALIGN(4)
  {
    __bi_entries_start = .;
    KEEP(*(.bi_entries));
    . = ALIGN(4);
    __bi_entries_end = .;
  } > FLASH
} INSERT AFTER .text;

/* Boot ROM trailer at end of image, after all RAM init sections are placed. */
SECTIONS
{
  .end_block : ALIGN(4)
  {
    __end_block_addr = .;
    KEEP(*(.end_block));
  } > FLASH
} INSERT AFTER .uninit;

/*------------------------------------------------------------------------------
 * Link-time parameters & exported symbols
 *----------------------------------------------------------------------------*/

/* Reserve total stack bytes at the top of RAM (tune to your worst-case).
 * This is a *guard* requested by cortex-m-rt. Keep generous for image loads. */
PROVIDE(_stack_size = 24K);

/* Heap ceiling: end of the last contiguous SRAM bank.
 * Rust code can compute: heap_size = _heap_ceiling - heap_start - safety_margin. */
PROVIDE(_heap_ceiling = ORIGIN(SRAM5) + LENGTH(SRAM5));

/* Optional: handy span for tooling */
PROVIDE(start_to_end = __end_block_addr - __start_block_addr);
PROVIDE(end_to_start = __start_block_addr - __end_block_addr);

/*------------------------------------------------------------------------------
 * Safety assertions
 * Ensure our “contiguous heap window” assumption holds at link time.
 *----------------------------------------------------------------------------*/
ASSERT(ORIGIN(SRAM4) == ORIGIN(RAM)  + LENGTH(RAM),  "SRAM4 must follow RAM contiguously");
ASSERT(ORIGIN(SRAM5) == ORIGIN(SRAM4) + LENGTH(SRAM4), "SRAM5 must follow SRAM4 contiguously");
