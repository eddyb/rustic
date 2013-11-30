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

use io;
use vga;
use mach;

static mut x: uint = 0;
static mut y: uint = 1;

static mut shifted: bool = false;

static mut led_state: u8 = 0;

// Scan code set #1
static scan_code_map: &'static str = "\
\x00\x1B1234567890-=\x08\tqwertyuiop[]\n?asdfghjkl;'`?\\zxcvbnm,./?*? ?????????????789-456+1230.?????";
static scan_code_map_shifted: &'static str = "\
\x00\x1B!@#$%^&*()_+\x08\tQWERTYUIOP{}\n?ASDFGHJKL:\"~?|ZXCVBNM<>??*? ?????????????789-456+1230.?????";


pub fn init() {
    // Put the keyboard into scan code set 1, ready for our mapping.
    /*
    kb_cmd_wait();
    io::outport(0x60, 0xF0u8);
    kb_cmd_wait();
    io::outport(0x60, 1u8);
    */

    mach::register_irq(1, || {
        // Check status, make sure a key is actually pending.
        if io::inport::<u8>(0x64) & 1 == 1 {

            // Get scancode.
            let scancode: u8 = io::inport(0x60);

            // Top bit set means 'key up'
            if scancode & 0x80 != 0 {
                let scancode = scancode & !0x80;
                match scancode {
                    0x2A | 0x36 => unsafe { shifted = false },
                    0x3A => leds(0b100), // Caps lock
                    0x45 => leds(0b010), // Number lock
                    0x46 => leds(0b001), // Scroll lock
                    _ => handle_key(scancode)
                }
            } else {
                match scancode {
                    0x2A | 0x36 => unsafe { shifted = true },
                    _ => {}
                }
            }
        }
    });
}

fn kb_cmd_wait() {
    loop {
        let status: u8 = io::inport(0x64);
        if status & 0x2 == 0 { break; }
    }
}

fn kb_data_wait() {
    loop {
        let status: u8 = io::inport(0x64);
        if status & 0x1 != 0 { break; }
    }
}

pub fn leds(state: u8) {
    unsafe { led_state ^= state; }

    kb_cmd_wait();
    io::outport(0x60, 0xEDu8);
    kb_data_wait();
    unsafe { io::outport(0x60, led_state); }
}

fn handle_key(scancode: u8) {
    // Sanity.
    if scancode > 0x58u8 { return; }

    let c: u8 = unsafe {
        if shifted {
            scan_code_map_shifted[scancode] as u8
        } else {
            scan_code_map[scancode] as u8
        }
    };
    let s: &str = unsafe { core::mem::transmute((&c, 1)) };

    unsafe {
        let off = vga::write(s, x, y, vga::White, vga::Black);

        // Update x/y
        y = off / 80;
        x = off % 80;

        if y >= vga::ROWS {
            y = vga::ROWS - 1;
        }
    }
}
