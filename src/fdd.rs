#![allow(unused)]
use crate::config::*;
use crate::mfm::*;
use core::arch::asm;
use teensycore::prelude::*;

pub fn fdd_read_index() -> u32 {
    return read_word(addrs::GPIO9) & (0x1 << 5);
}

#[derive(Clone, Copy)]
pub struct FloppyDriver {
    debug: bool,
    motor_active: bool,
}

impl FloppyDriver {
    pub fn new() -> Self {
        // Create a generic configuration for normal pins
        let generic_config: PadConfig = PadConfig {
            hysterisis: false,
            resistance: PullUpDown::PullUp100k,
            pull_keep: PullKeep::Pull,
            pull_keep_en: false,
            open_drain: false,
            speed: PinSpeed::Max200MHz,
            drive_strength: DriveStrength::Max,
            fast_slew_rate: true,
        };

        pin_pad_config(DRIVE_PIN, generic_config.clone());
        pin_pad_config(MOTOR_PIN, generic_config.clone());
        pin_pad_config(DIR_PIN, generic_config.clone());
        pin_pad_config(STEP_PIN, generic_config.clone());
        pin_pad_config(WRITE_PIN, generic_config.clone());
        pin_pad_config(GATE_PIN, generic_config.clone());
        pin_pad_config(HEAD_SEL_PIN, generic_config.clone());

        pin_mode(DRIVE_PIN, Mode::Output);
        pin_mode(MOTOR_PIN, Mode::Output);
        pin_mode(DIR_PIN, Mode::Output);
        pin_mode(STEP_PIN, Mode::Output);
        pin_mode(HEAD_SEL_PIN, Mode::Output);
        pin_mode(WRITE_PIN, Mode::Output);
        pin_mode(GATE_PIN, Mode::Output);

        pin_out(DRIVE_PIN, Power::High);
        pin_out(MOTOR_PIN, Power::High);
        pin_out(DIR_PIN, Power::High);
        pin_out(STEP_PIN, Power::High);
        pin_out(HEAD_SEL_PIN, Power::High);
        pin_out(WRITE_PIN, Power::High);
        pin_out(GATE_PIN, Power::High);

        // Create a generic configuration for pullup resistors
        let pullup_config: PadConfig = PadConfig {
            hysterisis: false,
            resistance: PullUpDown::PullUp22k,
            pull_keep: PullKeep::Pull,
            pull_keep_en: true,
            open_drain: true,
            speed: PinSpeed::Max200MHz,
            drive_strength: DriveStrength::MaxDiv3,
            fast_slew_rate: true,
        };

        pin_pad_config(INDEX_PIN, pullup_config.clone());
        pin_pad_config(TRACK00_PIN, pullup_config.clone());
        pin_pad_config(WRITE_PROTECT_PIN, pullup_config.clone());
        pin_pad_config(DISK_CHANGE_PIN, pullup_config.clone());

        // Read pin specifically
        pin_pad_config(
            READ_PIN,
            PadConfig {
                hysterisis: false,
                resistance: PullUpDown::PullUp47k,
                pull_keep: PullKeep::Pull,
                pull_keep_en: true,
                open_drain: true,
                speed: PinSpeed::Max200MHz,
                drive_strength: DriveStrength::Max,
                fast_slew_rate: true,
            },
        );

        // Set them to outputs
        pin_mode(INDEX_PIN, Mode::Input);
        pin_mode(TRACK00_PIN, Mode::Input);
        pin_mode(WRITE_PROTECT_PIN, Mode::Input);
        pin_mode(READ_PIN, Mode::Input);
        pin_mode(DISK_CHANGE_PIN, Mode::Input);

        return FloppyDriver {
            debug: true,
            motor_active: false,
        };
    }

    fn soft_reset(&mut self) {
        pin_out(DRIVE_PIN, Power::High);
        pin_out(MOTOR_PIN, Power::High);
        pin_out(DIR_PIN, Power::High);
        pin_out(STEP_PIN, Power::High);
        pin_out(WRITE_PIN, Power::High);
        pin_out(GATE_PIN, Power::High);
        pin_out(HEAD_SEL_PIN, Power::High);

        self.motor_active = false;
        wait_exact_ns(MS_TO_NANO * 500);
    }

    pub fn motor_on(&mut self, on: bool) {
        if self.motor_active == on {
            return;
        }

        self.motor_active = on;

        if on {
            debug_str(b"Power cycling...");
            pin_out(GATE_PIN, Power::High);
            pin_out(DRIVE_PIN, Power::High);
            pin_out(HEAD_SEL_PIN, Power::High);
            pin_out(MOTOR_PIN, Power::High);
            wait_exact_ns(MS_TO_NANO * 3000);
            pin_out(DRIVE_PIN, Power::Low);
            pin_out(HEAD_SEL_PIN, Power::High);
            pin_out(MOTOR_PIN, Power::Low);
            wait_exact_ns(MS_TO_NANO * 1000);
        } else {
            pin_out(MOTOR_PIN, Power::High);
        }

        if !on {
            debug_str(b"Shutting down motor");
            return;
        }

        debug_str(b"Cycle the power...");
        wait_exact_ns(MS_TO_NANO * 6000);

        debug_str(b"Spinning up motor");
        debug_str(b"Waiting for index pulse...");

        // Do a step

        let start = nanos();
        while pin_read_fast!(INDEX_PIN) > 0 && (nanos() - start) < 10000 * MS_TO_NANO {
            assembly!("nop");
        }

        if pin_read_fast!(INDEX_PIN) == 0 {
            debug_str(b"Received index pulse!");
            wait_exact_ns(MS_TO_NANO * 5000);
        } else {
            debug_str(b"Did not receive index pulse");
            self.motor_active = false;
        }
    }

    pub fn step(&self, dir: Power, times: u8) {
        pin_out(DIR_PIN, dir);
        for _ in 0..times {
            pin_out(STEP_PIN, Power::High);
            wait_exact_ns(MS_TO_NANO * 11);
            pin_out(STEP_PIN, Power::Low);
            wait_exact_ns(MS_TO_NANO * 11);
            pin_out(STEP_PIN, Power::High);
        }
    }

    #[no_mangle]
    pub fn read_track(&mut self) {
        let mut sectors = 0;
        let mut buf: [u8; 512] = [0; 512];

        while fdd_read_index() == 0 {
            assembly!("nop");
        }

        while fdd_read_index() > 0 {
            assembly!("nop");
        }

        while fdd_read_index() == 0 {
            assembly!("nop");
        }

        let start = nanos() / MS_TO_NANO;

        while fdd_read_index() > 0 {
            if mfm_sync() {
                mfm_read_bytes(&mut buf);
                debug_str(b"===========");
                debug_hex(buf[0] as u32, b"buf[0]");
                debug_hex(buf[1] as u32, b"buf[1]");
                debug_hex(buf[2] as u32, b"buf[2]");
                debug_hex(buf[3] as u32, b"buf[3]");

                sectors += 1;
            }
        }

        let end = nanos() / MS_TO_NANO;

        debug_u64(sectors as u64, b"Sectors found");
        debug_u64((end - start) as u64, b"TIMING");
    }

    pub fn begin(&mut self) {
        self.soft_reset();
        self.motor_on(true);
    }

    pub fn seek_track00(&mut self) -> Option<usize> {
        self.motor_on(true);
        let mut cycles: usize = 0;

        for _ in 0..100 {
            if pin_read_fast!(TRACK00_PIN) == 0 {
                return Some(cycles);
            }

            cycles += 1;
            self.step(Power::High, 1);
        }

        for _ in 0..20 {
            if pin_read_fast!(TRACK00_PIN) == 0 {
                return Some(cycles);
            }

            cycles += 1;
            self.step(Power::Low, 1);
        }

        return None;
    }

    pub fn measure_sector(&self) {}
    pub fn end(&self) {}
}
