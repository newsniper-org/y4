/*
 * SPDX-License-Identifier: Apache-2.0
 * SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
 *
 * Y4 root task link layout.
 *
 * seL4 loads the root task ELF into a region it picks at boot.  The
 * ELF's program headers tell seL4 the virtual address each segment
 * wants — we pick 0x40_0000 (4 MiB) which sits well below the seL4
 * x86_64 user-top (0x00007ffffffff000) and well above the first MiB
 * (BIOS / VGA / multiboot scratch).
 */

ENTRY(_start)

SECTIONS
{
    . = 0x400000;

    .text : ALIGN(4K) {
        *(.text._start)
        *(.text .text.*)
    }

    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K) {
        *(.data .data.*)
    }

    .bss : ALIGN(4K) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /* Drop sections we don't need on bare metal. */
    /DISCARD/ : {
        *(.comment)
        *(.note .note.*)
        *(.eh_frame)
        *(.eh_frame_hdr)
    }
}
