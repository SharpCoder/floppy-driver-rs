#![allow(internal_features)]
#![feature(lang_items)]
#![crate_type = "staticlib"]
#![no_std]

use teensycore::prelude::*;

mod fdd;
use fdd::*;

teensycore::main!({
    // Create the floppy driver
    let mut driver = FloppyDriver::new(18, 19, 20, 21, 22, 8, 9, 10, 11, 12, 13, 14);
    driver.begin();

    loop {
        driver.motor_on(true);

        driver.read_track();

        // match driver.seek_track00() {
        //     Some(cycles) => {
        //         print(b"Found track0 in ");
        //         print_u32(cycles as u32);
        //         print(b" cycles!\n");

        //         // Must wait a bit after the last pulse
        //         wait_exact_ns(MS_TO_NANO * 20);

        //         driver.read_track();
        //     }
        //     None => {
        //         driver.motor_on(false);
        //         debug_str(b"Did not find tack00\n");
        //     }
        // }

        wait_exact_ns(MS_TO_NANO * 5000);
    }
});
