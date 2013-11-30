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
use serial;

use io;
use cpu;

type handlers = [Handler, ..16];

struct Handler {
    f: 'static||,
    set: bool,
    level: bool,
}

static REMAP_BASE: u8 = 0x20;

static mut irq_handlers: *mut handlers = 0 as *mut handlers;

pub fn init() {
    io::outport(0x20, 0x11u8);
    io::outport(0xA0, 0x11u8);
    io::outport(0x21, REMAP_BASE); // Remap to start at the remap base.
    io::outport(0xA1, REMAP_BASE + 8u8);
    io::outport(0x21, 0x04u8);
    io::outport(0xA1, 0x02u8);
    io::outport(0x21, 0x01u8);
    io::outport(0xA1, 0x01u8);

    // Mask all, machine layer will call our enable() when an IRQ is registered.
    io::outport(0x21, 0xFFu8);
    io::outport(0xA1, 0xFFu8);

    // Allocate space for our handler list.
    unsafe { irq_handlers = core::heap::malloc(192) as *mut handlers; }

    // Set handlers, set IRQ entries on the CPU.
    let mut i = 0;
    while i < 16 {
        unsafe { (*irq_handlers)[i].set = false; }
        cpu::register_trap(i + REMAP_BASE as uint, irq);
        i += 1;
    }
}

pub fn register(n: uint, f: 'static||) {
    // TODO: expose level-trigger Boolean
    unsafe {
        (*irq_handlers)[n].f = f;
        (*irq_handlers)[n].set = true;
        (*irq_handlers)[n].level = true;
    }
}

pub fn enable(line: uint) {
    if line > 7 {
        let actual = line - 8;
        let curr: u8 = io::inport(0xA1);
        io::outport(0xA1, curr & !((1 << actual) as u8))
    } else {
        let curr: u8 = io::inport(0x21);
        io::outport(0x21, curr & !((1 << line) as u8))
    }
}

pub fn disable(line: uint) {
    if line > 7 {
        let actual = line - 8;
        let curr: u8 = io::inport(0xA1);
        io::outport(0xA1, curr | ((1 << actual) as u8))
    } else {
        let curr: u8 = io::inport(0x21);
        io::outport(0x21, curr | ((1 << line) as u8))
    }
}

fn eoi(n: uint) {
    if n > 7 { io::outport(0xA0, 0x20u8); }
    io::outport(0x20, 0x20u8);
}

fn irq(n: uint) {
    let n = n - REMAP_BASE as uint;

    // Get status registers for master/slave
    io::outport(0x20, 0x0Bu8);
    io::outport(0xA0, 0x0Bu8);
    let slave_isr: u8 = io::inport(0xA0);
    let master_isr: u8 = io::inport(0x20);
    let status = (slave_isr as u16 << 8) | master_isr as u16;

    // Spurious IRQ?
    if n == 7 {
        if (status & (1 << 7)) == 0 {
            serial::write("spurious IRQ 7\n");
            return;
        }
    } else if n == 15 {
        if (status & (1 << 15)) == 0 {
            serial::write("spurious IRQ 15\n");
            eoi(7);
            return;
        }
    }

    if (status & (1 << n)) == 0 {
        serial::write("IRQ stub called with no interrupt status");
        return;
    }

    // Get the handler we need.
    let h = unsafe { &(*irq_handlers)[n] };
    if h.set == true {
        // Edge triggered IRQs need to be ACKed before the handler.
        if h.level == false {
            eoi(n);
        }

        // Handle!
        (h.f)();

        // ACK level triggered IRQ.
        if h.level == true {
            eoi(n);
        }
    } else {
        // Unhandled IRQ, just send the EOI and hope all's well.
        serial::write("Unhandled IRQ");
        eoi(n);
    }
}

