#![allow(internal_features)]
#![feature(global_asm)]
#![feature(lang_items)]
#![crate_type = "staticlib"]
#![no_std]

mod config;
mod fdd;
mod mfm;
mod timing;

use core::arch::asm;
use fdd::*;
use teensycore::prelude::*;
use timing::*;

#[cfg(feature = "testing")]
extern crate std;

#[cfg(not(feature = "testing"))]
teensycore::main!({
    wait_exact_ns(MS_TO_NANO * 2000);

    // loop {
    //     unsafe {
    //         let start = nanos() / MS_TO_NANO;
    //         pin_out(13, Power::High);
    //         wait_cycle(F_CPU * 5);
    //         let end = nanos() / MS_TO_NANO;
    //         debug_u64((end - start) as u64, b"Timing");
    //         pin_out(13, Power::Low);
    //         wait_cycle(F_CPU * 5);
    //     }
    // }

    // Create the floppy driver
    fdd_init();

    wait_exact_ns(MS_TO_NANO * 2000);

    match fdd_read_write_protect() {
        true => debug_str(b"Media is write protected"),
        false => debug_str(b"Media is not write protected"),
    }

    wait_exact_ns(MS_TO_NANO * 1000);

    loop {
        fdd_set_motor(true);

        match fdd_seek_track00() {
            Some(cycles) => {
                print(b"Found track0 in ");
                print_u32(cycles as u32);
                print(b" cycles!\n");

                let head = 0;
                let cylinder = 10;
                let sector = 12;

                // Write a sector
                debug_str(b"Beginning write seek...");
                if fdd_write_sector(head, cylinder, sector, &[0x10, 0x20, 0x30, 0x40]) {
                    debug_str(b"Write complete!");
                    // Read a sector
                    match fdd_read_sector(head, cylinder, sector) {
                        None => {
                            debug_str(b"Failed to find sector");
                        }
                        Some(sector) => {
                            debug_str(b"Found the sector!!");

                            // Dump some bytes
                            for i in 0..20 {
                                debug_hex(sector.data[i] as u32, b"");
                                wait_exact_ns(MS_TO_NANO);
                            }
                        }
                    }
                } else {
                    debug_str(b"Failed to write");
                }
            }
            None => {
                debug_str(b"Did not find tack00");
            }
        }

        debug_str(b"Entering sleep mode...");
        fdd_shutdown();

        loop {
            assembly!("nop");
        }
    }
});
