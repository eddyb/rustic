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

use core;
use core::mem::size_of;

type IdtTable = [Entry, ..256];

// One handler per interrupt line.
type IdtHandlers = [Handler, ..256];

// Base for all our IRQ handling.
extern "C" { fn isrs_base(); }

// Size of the interrupt stub, so we can create our initial IDT easily.
static ISR_STUB_LENGTH: uint = 10;

#[packed]
struct Register {
    limit: u16,
    addr: *IdtTable,
}

#[packed]
struct Entry {
    handler_low: u16,
    selector: u16,
    always0: u8,
    flags: u8,
    handler_high: u16,
}

struct Handler {
    f: 'static|uint|,
    set: bool,
}

struct Table {
    reg: *mut Register,
    table: *mut IdtTable,
    handlers: *mut IdtHandlers,
}

impl Register {
    pub fn new(idt: *IdtTable) -> Register {
        Register {
            addr: idt,
            limit: (size_of::<IdtTable>() + 1) as u16,
        }
    }
}

impl Entry {
    pub fn new(handler: uint, sel: u16, flags: u8) -> Entry {
        Entry {
            handler_low: (handler & 0xFFFF) as u16,
            selector: sel,
            always0: 0,
            flags: flags | 0x60,
            handler_high: ((handler >> 16) & 0xFFFF) as u16,
        }
    }
}

static mut system_idt: Table = Table {
    table: 0 as *mut IdtTable,
    reg: 0 as *mut Register,
    handlers: 0 as *mut IdtHandlers,
};

fn entry(index: uint, handler: uint, sel: u16, flags: u8) {
    unsafe {
        (*system_idt.table)[index] = Entry::new(handler, sel, flags)
    }
}

pub fn register(index: uint, handler: 'static|uint|) {
    unsafe {
        (*system_idt.handlers)[index].f = handler;
        (*system_idt.handlers)[index].set = true;
    }
}

pub fn init() {
    unsafe {
        system_idt.table = core::heap::malloc(2048) as *mut IdtTable;
        system_idt.reg = core::heap::malloc(6) as *mut Register;
        system_idt.handlers = core::heap::malloc(2048) as *mut IdtHandlers;
        *system_idt.reg = Register::new(system_idt.table as *IdtTable);
    }

    // Load default IDT entries, that generally shouldn't ever be changed.
    let mut i = 0;
    let mut base = isrs_base as uint;
    while i < 256 {
        entry(i, base, 0x08u16, 0x8E);
        unsafe { (*system_idt.handlers)[i].set = false; }
        base += ISR_STUB_LENGTH;
        i += 1;
    }
}

#[no_mangle]
pub extern "C" fn isr_rustentry(which: uint) {
    // Entry point for IRQ - find if we have a handler configured or not.
    let x = unsafe { &(*system_idt.handlers)[which] };
    if x.set == true {
        (x.f)(which);
    }
}

pub fn load() {
    unsafe { asm!("lidt ($0)" :: "r" (system_idt.reg)); }
}

