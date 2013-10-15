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

//BEGIN Multiboot

enum MultibootFlags {
    Align   = 1<<0,
    MemInfo = 1<<1
};

#define MAGIC 0x1BADB002
#define FLAGS (Align | MemInfo)

struct MultibootHeader {
    unsigned magic, flags, checksum;
};

struct MultibootHeader multiboot_header __attribute__ ((aligned(4), section(".multiboot"))) = {
    .magic = MAGIC,
    .flags = FLAGS,
    .checksum = -(MAGIC + FLAGS)
};

//END Multiboot

/// Required by zero.rs, and needs to be more noisy.
__attribute__((noreturn)) void abort() {
    while(1)
        asm volatile("cli; hlt");
}

/// Required by Rust. \todo Needs to be implemented!
void __morestack() {
    abort();
}

static unsigned char stack[128 * 1024];

/// Rust entry point.
extern int main(int, char **);

/// Kernel entry point.
__attribute__((naked)) void _start() {
    // TODO: don't use %gs:30 - fix the target?
    // This is required as Rust's main() at least checks to see
    // if the stack needs to be expanded in its prologue. If it
    // decides it does, it calls __morestack.
    // Still need to identify a much better solution than this.
    // Set up stack.
    asm("movl %0, %%esp" :: "i" (stack + sizeof(stack)));
    asm("movl %0, %%gs:0x30" :: "i" (stack));

    main(0, 0);

    abort();
}
