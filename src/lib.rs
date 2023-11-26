#![allow(internal_features)]
#![feature(lang_items)]
#![crate_type = "staticlib"]
#![no_std]

mod config;
mod fdd;
mod mfm;

use core::arch::asm;
use fdd::*;
use teensycore::prelude::*;

#[cfg(feature = "testing")]
extern crate std;

#[cfg(not(feature = "testing"))]
teensycore::main!({
    // Create the floppy driver
    fdd_init();

    loop {
        fdd_set_motor(true);

        match fdd_seek_track00() {
            Some(cycles) => {
                print(b"Found track0 in ");
                print_u32(cycles as u32);
                print(b" cycles!\n");

                // Write a sector
                // fdd_write_sector(0, 18, 2, &[1, 2, 3, 1, 2, 3, 1, 2, 3, 4]);

                // Read a sector
                match fdd_read_sector(0, 4, 3) {
                    None => {
                        debug_str(b"Failed to find sector");
                    }
                    Some(sector) => {
                        debug_str(b"Found the sector!!");

                        // Dump the first 50 bytes
                        for i in 0..10 {
                            debug_hex(sector.data[i] as u32, b"");
                            wait_exact_ns(MS_TO_NANO);
                        }
                    }
                }

                fdd_shutdown();

                loop {
                    assembly!("nop");
                }
            }
            None => {
                fdd_set_motor(false);
                debug_str(b"Did not find tack00\n");
            }
        }

        wait_exact_ns(MS_TO_NANO * 5000);
    }
});
