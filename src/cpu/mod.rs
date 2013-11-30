/*
 * Copyright (c) 2013 Matthew Iselin
 *
 * Permission to use, copy, modify, and distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */

use serial;

mod gdt;
// NOTE public so extern "C" fn isr_rustentry is external.
pub mod idt;

// External variable in assembly code (not actually a function)
extern { fn tls_emul_segment(); }

pub fn init() {
    // Configure and load GDT
    gdt::init();
    gdt::entry(0, 0, 0, 0, 0); // 0x00 - NULL
    gdt::entry(1, 0, 0xFFFFFFFF, 0x98, 0xCF); // 0x08 - Kernel Code
    gdt::entry(2, 0, 0xFFFFFFFF, 0x92, 0xCF); // 0x10 - Kernel Data
    gdt::entry(3, 0, 0xFFFFFFFF, 0xF8, 0xCF); // 0x18 - User Code
    gdt::entry(4, 0, 0xFFFFFFFF, 0xF2, 0xCF); // 0x20 - User Data
    gdt::entry(5, tls_emul_segment as uint, 0xFFFFFFFF, 0x92, 0xCF); // 0x28 - TLS emulation (for stack switching support)
    gdt::load(0x08, 0x10, 0x28);

    // Configure and load IDT; don't enable IRQs until machine init is done.
    idt::init();
    idt::load();

    // Load #PF handler now.
    register_trap(14, |_: uint| {
        serial::write("BUS ERROR");
        loop {}
    });
}

pub fn register_trap(trap: uint, f: 'static|uint|) {
    idt::register(trap, f);
}

pub fn set_interrupts(state: bool) {
    if(state == true) {
        unsafe { asm!("sti") }
    } else {
        unsafe { asm!("cli") }
    }
}

pub fn wait_for_interrupt() {
    set_interrupts(true);
    unsafe { asm!("hlt") }
}
