#![allow(internal_features)]
#![feature(lang_items)]
#![crate_type = "staticlib"]
#![no_std]

pub mod config;
pub mod fdd;
pub mod mfm;

use fdd::*;
use teensycore::prelude::*;

teensycore::main!({
    // Create the floppy driver
    let mut driver = FloppyDriver::new();
    driver.begin();

    loop {
        driver.motor_on(true);

        // driver.read_track();

        match driver.seek_track00() {
            Some(_cycles) => {
                // driver.step(Power::Low, 8);

                // print(b"Found track0 in ");
                // print_u32(cycles as u32);
                // print(b" cycles!\n");

                // Must wait a bit after the last pulse
                // wait_exact_ns(MS_TO_NANO * 20);
                driver.read_track();
            }
            None => {
                driver.motor_on(false);
                debug_str(b"Did not find tack00\n");
            }
        }

        wait_exact_ns(MS_TO_NANO * 5000);
    }
});
