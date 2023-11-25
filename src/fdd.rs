#![allow(unused)]
use crate::config::*;
use crate::mfm::*;
use core::arch::asm;
use teensycore::prelude::*;

/**
 * This is a total hack. It reads directly from the gpio register for pin 3.
 * Bypassing the pin_read method of teensycore because it's too slow.
 */
pub fn fdd_read_index() -> u32 {
    return read_word(addrs::GPIO9) & (0x1 << 5);
}

/**
 * Initialize the floppy driver. Configuring pull-ups and
 * setting a default value.
 */
pub fn fdd_init() {
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
    pin_pad_config(READ_PIN, pullup_config.clone());

    // Set them to outputs
    pin_mode(INDEX_PIN, Mode::Input);
    pin_mode(TRACK00_PIN, Mode::Input);
    pin_mode(WRITE_PROTECT_PIN, Mode::Input);
    pin_mode(READ_PIN, Mode::Input);
    pin_mode(DISK_CHANGE_PIN, Mode::Input);
}

/**
 * Change the state of the motor.
 */
pub fn fdd_set_motor(on: bool) {
    let motor_active = pin_read(MOTOR_PIN);

    // If the motor is unchanged, don't do anything
    if motor_active > 0 && !on || motor_active == 0 && on {
        return;
    }

    if on {
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
    while fdd_read_index() > 0 && (nanos() - start) < 10000 * MS_TO_NANO {
        assembly!("nop");
    }

    if fdd_read_index() == 0 {
        debug_str(b"Received index pulse!");
        wait_exact_ns(MS_TO_NANO * 5000);
    } else {
        debug_str(b"Did not receive index pulse");
        pin_out(MOTOR_PIN, Power::High);
    }
}

/**
 * Change the active track.
 */
pub fn fdd_step_track(dir: Power, times: usize) {
    pin_out(DIR_PIN, dir);
    for _ in 0..times {
        pin_out(STEP_PIN, Power::High);
        wait_exact_ns(MS_TO_NANO * 11);
        pin_out(STEP_PIN, Power::Low);
        wait_exact_ns(MS_TO_NANO * 11);
        pin_out(STEP_PIN, Power::High);
    }
}

/**
 * Seek to track 0.
 */
pub fn fdd_seek_track00() -> Option<usize> {
    fdd_set_motor(true);
    let mut cycles: usize = 0;

    for _ in 0..100 {
        if pin_read(TRACK00_PIN) == 0 {
            return Some(cycles);
        }

        cycles += 1;
        fdd_step_track(Power::High, 1);
    }

    for _ in 0..20 {
        if pin_read(TRACK00_PIN) == 0 {
            return Some(cycles);
        }

        cycles += 1;
        fdd_step_track(Power::Low, 1);
    }

    return None;
}

/**
 * Turn off the motor and soft reset.
 */
pub fn fdd_shutdown() {
    pin_out(DRIVE_PIN, Power::High);
    pin_out(MOTOR_PIN, Power::High);
    pin_out(DIR_PIN, Power::High);
    pin_out(STEP_PIN, Power::High);
    pin_out(WRITE_PIN, Power::High);
    pin_out(GATE_PIN, Power::High);
    pin_out(HEAD_SEL_PIN, Power::High);
    wait_exact_ns(MS_TO_NANO * 500);
}
