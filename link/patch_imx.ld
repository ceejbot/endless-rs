/* Linker script for Endless patches built with Rust.
 * Adapted from FxPatchSDK/internal/patch_imx.ld.
 *
 * Places the patch binary at a fixed RAM address with the PatchHeader
 * at offset 0, followed by code/data, then zero-initialized BSS.
 */

_PATCH_BASE = DEFINED(PATCH_LOAD_ADDR) ? PATCH_LOAD_ADDR : 0x80000000;

MEMORY
{
  RAM (rxw) : ORIGIN = _PATCH_BASE, LENGTH = 512K
}

ENTRY(patch_header)

SECTIONS
{
  /* Header at the very start — emitted by global_asm! in the macro */
  .patch_header ORIGIN(RAM) : ALIGN(4)
  {
    KEEP(*(.patch_header))
  } > RAM

  /* Code and initialized data follow contiguously */
  .patch_image : ALIGN(4)
  {
    __patch_start = .;
    *(.text .text.*)
    *(.rodata .rodata.*)
    . = ALIGN(4);
    *(.data .data.*)
    __patch_end = .;
  } > RAM

  /* Zero-initialized data (not in the binary file) */
  .patch_bss (NOLOAD) : ALIGN(4)
  {
    __patch_bss_start = .;
    *(.bss .bss.*)
    *(COMMON)
    __patch_bss_end = .;
  } > RAM

  /DISCARD/ :
  {
    *(.comment*)
    *(.note*)
    *(.ARM.exidx*)
    *(.ARM.extab*)
    *(.eh_frame*)
    *(.eh_frame_hdr*)
  }

  /* Pre-computed absolute symbols referenced by global_asm! in the macro.
   * The LLVM integrated assembler cannot evaluate cross-section subtractions,
   * so we let the linker compute them here. */
  patch_image_size = __patch_end - __patch_start;
  patch_bss_size = __patch_bss_end - __patch_bss_start;
}
