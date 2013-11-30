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

type GdtTable = [Entry, ..16];

#[packed]
struct Register {
    limit: u16,
    addr: *GdtTable,
}

#[packed]
struct Entry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    gran: u8,
    base_high: u8,
}

struct Table {
    reg: *mut Register,
    table: *mut GdtTable
}

impl Register {
    pub fn new(gdt: *GdtTable) -> Register {
        Register {
            addr: gdt,
            limit: (size_of::<GdtTable>() + 1) as u16,
        }
    }
}

impl Entry {
    pub fn new(base: uint, limit: uint, access: u8, gran: u8) -> Entry {
        Entry {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_mid: ((base >> 16) & 0xFF) as u8,
            access: access,
            gran: gran,
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

static mut system_gdt: Table = Table {
    table: 0 as *mut GdtTable,
    reg: 0 as *mut Register,
};

pub fn init() {
    unsafe {
        system_gdt.table = core::heap::malloc(128) as *mut GdtTable;
        system_gdt.reg = core::heap::malloc(6) as *mut Register;
        *system_gdt.reg = Register::new(system_gdt.table as *GdtTable);
    }
}

pub fn load(code_seg: u16, data_seg: u16, tls_emul_seg: u16) {
    unsafe { asm!("
        lgdt ($0);
        jmp $1, $$.g;
        .g:
        mov $2, %ax;
        mov %ax, %ds;
        mov %ax, %es;
        mov %ax, %fs;
        mov %ax, %ss;
        mov $3, %ax;
        mov %ax, %gs;" :: "r" (system_gdt.reg), "Ir" (code_seg), "Ir" (data_seg), "Ir" (tls_emul_seg) : "ax"); }
}

pub fn entry(index: int, base: uint, limit: uint, access: u8, gran: u8) {
    unsafe {
        (*system_gdt.table)[index] = Entry::new(base, limit, access, gran);
    }
}

