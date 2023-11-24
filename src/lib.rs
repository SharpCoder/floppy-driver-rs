#![allow(internal_features)]
#![feature(lang_items)]
#![crate_type = "staticlib"]
#![no_std]

use teensycore::prelude::*;

mod fdd;
use fdd::*;

teensycore::main!({
    // Create the floppy driver
    let mut driver = FloppyDriver::new(FloppyConfiguration {
        index_pin: 3,
        drive_pin: 4,
        motor_pin: 5,
        dir_pin: 6,
        step_pin: 7,
        write_pin: 8,
        gate_pin: 9,
        track00_pin: 10,
        write_protect_pin: 11,
        read_pin: 12,
        head_sel_pin: 14,
        disk_change_pin: 15,
    });
    driver.begin();

    loop {
        driver.motor_on(true);

        // driver.read_track();

        match driver.seek_track00() {
            Some(_cycles) => {
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
