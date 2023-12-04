#![allow(internal_features)]
#![feature(lang_items)]
#![crate_type = "staticlib"]
#![no_std]

mod config;
mod fdd;
mod mfm;

use core::arch::asm;
use fdd::*;
use mfm::mfm_dump_stats;
use teensycore::prelude::*;

#[cfg(feature = "testing")]
extern crate std;

#[cfg(not(feature = "testing"))]
teensycore::main!({
    wait_exact_ns(MS_TO_NANO * 3000);

    fdd_init();
    fdd_set_motor(true);

    wait_exact_ns(MS_TO_NANO * 2000);

    match fdd_read_write_protect() {
        true => debug_str(b"Media is not write protected"),
        false => debug_str(b"Media is write protected"),
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
                let cylinder = 7;
                let sector = 2;

                mfm_dump_stats();

                // // Write a sector
                // debug_str(b"Beginning write seek...");
                // if fdd_write_sector(head, cylinder, sector, &[0x55; 512]) {
                //     debug_str(b"Write complete!");
                // } else {
                //     debug_str(b"Failed to write");
                // }

                // Read a sector
                match fdd_read_sector(head, cylinder, sector) {
                    None => {
                        debug_str(b"Failed to find sector");
                    }
                    Some(sector) => {
                        debug_str(b"Found the sector!!");

                        // Dump some bytes
                        for i in 0..10 {
                            debug_hex(sector.data[i] as u32, b"");
                            wait_exact_ns(MS_TO_NANO);
                        }
                    }
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
