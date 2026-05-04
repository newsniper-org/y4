// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Y4 root task — Phase B step 5 milestone: `Hello, Y4` on the serial
//! console.
//!
//! Layout:
//!
//! - **`_start`** — entry point seL4 jumps to.  Sets up an initial
//!   stack at the top of `.bss` and calls [`rust_main`].
//! - **[`rust_main`]** — calls [`debug_put_string`] with the greeting
//!   then enters an idle loop (root task must not return; doing so
//!   delivers a fault to seL4 which would panic the kernel).
//! - **[`debug_put_char`]** — invokes the seL4 `SysDebugPutChar = -9`
//!   syscall via inline asm.  Available because the kernel was built
//!   with `KernelDebugBuild = ON` and `KernelPrinting = ON`
//!   (`boot/x86_64-debug.cmake`).
//!
//! This is the *first* userland the seL4 base hands control to.  All
//! subsequent Y4 specialization (lease scheduler, capsule init, scheme
//! registry bootstrap) layers on top of this entry point in dependent
//! PRs.

#![no_std]
#![no_main]
#![allow(clippy::missing_safety_doc)]

use core::arch::{asm, naked_asm};
use core::panic::PanicInfo;

/// seL4 `SysDebugPutChar` syscall id (from
/// `build/sel4/x86_64-debug/gen_headers/arch/api/syscall.h`).
const SYS_DEBUG_PUT_CHAR: i64 = -9;

/// Root-task entry point.  Establishes an initial stack and tail-calls
/// [`rust_main`].  `naked` so no prologue clobbers `rdi` (which seL4
/// would put `seL4_BootInfo*` into for production root tasks).
///
/// Stack lives at the high end of `INITIAL_STACK` — a fixed 16 KiB
/// buffer in `.bss`.  Plenty for the greeting + halt loop; replaced
/// when the lease scheduler PR brings real thread setup.
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        // rsp = &INITIAL_STACK[STACK_SIZE]
        "lea rsp, [rip + {stack_top}]",
        "call {main}",
        // Should never return; if it does, halt forever.
        "2: hlt",
        "jmp 2b",
        stack_top = sym INITIAL_STACK_TOP,
        main = sym rust_main,
    );
}

const STACK_SIZE: usize = 16 * 1024;

#[unsafe(no_mangle)]
static mut INITIAL_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

/// `_start` loads `rsp` from this symbol — see linker layout below.
/// Resolved at link time to `&INITIAL_STACK[STACK_SIZE]`.
#[unsafe(no_mangle)]
static INITIAL_STACK_TOP: [u8; 0] = [];

/// Rust-side root-task main.  Prints the milestone greeting and
/// halts.
#[unsafe(no_mangle)]
extern "C" fn rust_main() -> ! {
    // Adjust `INITIAL_STACK_TOP` to actually point at the top of the
    // stack buffer.  We can't do this at link time without a custom
    // symbol, so do it once on entry: bump `rsp` past the stack.
    unsafe {
        let top = core::ptr::addr_of!(INITIAL_STACK).cast::<u8>().add(STACK_SIZE);
        asm!("mov rsp, {0}", in(reg) top, options(nostack, preserves_flags));
    }
    debug_put_string(b"Hello, Y4\n");
    loop {
        // `hlt` is ring-0 only; in ring-3 it would GP-fault and seL4
        // would restart the root task, looping the greeting forever.
        // `pause` is ring-3 safe and gives the CPU a back-off hint.
        unsafe { asm!("pause", options(nomem, nostack, preserves_flags)); }
    }
}

/// Issue one `seL4_DebugPutChar` syscall.
#[inline]
fn debug_put_char(c: u8) {
    unsafe {
        asm!(
            "mov r12, rsp",
            "syscall",
            "mov rsp, r12",
            in("rdx") SYS_DEBUG_PUT_CHAR,
            in("rdi") u64::from(c),
            in("rsi") 0_u64,
            out("rcx") _,
            out("r11") _,
            out("r12") _,
            options(nostack, preserves_flags),
        );
    }
}

fn debug_put_string(s: &[u8]) {
    for &b in s {
        debug_put_char(b);
    }
}

/// Panic handler — print marker + halt.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    debug_put_string(b"\n[y4-roottask] PANIC\n");
    loop {
        unsafe { asm!("pause", options(nomem, nostack, preserves_flags)); }
    }
}
