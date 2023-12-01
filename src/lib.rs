#![allow(internal_features)]
#![feature(lang_items)]
#![crate_type = "staticlib"]
#![no_std]

mod config;
mod fdd;
mod mfm;

use core::arch::asm;
use fdd::*;
use mfm::{mfm_dump_stats, Symbol};
use teensycore::prelude::*;

#[cfg(feature = "testing")]
extern crate std;

#[cfg(not(feature = "testing"))]
teensycore::main!({
    wait_exact_ns(MS_TO_NANO * 2000);

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

                // 10 is ruined
                let head = 0;
                let cylinder = 11;
                let sector = 9;

                mfm_dump_stats();

                // Wait for a barrier
                if mfm::mfm_sync() {
                    debug_str(b"Found barrier!");
                } else {
                    debug_str(b"Did not find a barrier");
                    fdd_shutdown();
                    loop {}
                }

                let mut flux_signals: [Symbol; 4096] = [Symbol::Pulse10; 4096];
                const FLUX_COUNT: usize = 40;
                // match fdd_read_sector(head, cylinder, sector) {
                //     None => {
                //         debug_str(b"Failed to find sector");
                //     }
                //     Some(sector) => {
                //         debug_str(b"Found the sector!!");

                //         // Dump some bytes
                //         for i in 0..10 {
                //             debug_hex(sector.data[i] as u32, b"");
                //             wait_exact_ns(MS_TO_NANO);
                //         }
                //     }
                // }

                // // Write a sector
                debug_str(b"Beginning write seek...");
                if fdd_write_sector(head, cylinder, sector, &[0xFB, 0x13, 0x37, 0xA1, 0, 0]) {
                    debug_str(b"Write complete!");
                    // Debug a sector
                    match fdd_debug_sector(head, cylinder, sector, &mut flux_signals, FLUX_COUNT) {
                        false => {
                            debug_str(b"Failed to find sector after write operation");
                        }
                        _ => {
                            let mut pulses: [u8; FLUX_COUNT] = [0; FLUX_COUNT];
                            for i in 0..FLUX_COUNT {
                                match flux_signals[i] {
                                    Symbol::Pulse10 => {
                                        pulses[i] = b'S';
                                    }
                                    Symbol::Pulse100 => {
                                        pulses[i] = b'M';
                                    }
                                    Symbol::Pulse1000 => {
                                        pulses[i] = b'L';
                                    }
                                }
                            }

                            debug_str(&pulses);
                        }
                    }

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
