#![allow(unused)]

use crate::config::*;
use crate::mfm::*;
use core::arch::asm;
use teensycore::prelude::*;

static mut FLOPPY_SIDE: u8 = 0;
static mut FLOPPY_TRACK: u8 = 0;
static mut FLOPPY_MOTOR_ON: bool = false;

#[repr(C)]
pub struct SectorID {
    pub id: u8,
    pub cylinder: u8,
    pub head: u8,
    pub sector: u8,
    pub size: u8,
    pub crc1: u16,
    pub data: [u8; 512],
    pub crc2: u16,
}

impl SectorID {
    pub fn new() -> Self {
        return SectorID {
            id: 0,
            cylinder: 0,
            head: 0,
            sector: 0,
            size: 0,
            crc1: 0,
            data: [0; 512],
            crc2: 0,
        };
    }
}

/**
 * This is a total hack. It reads directly from the gpio register for pin 3.
 * Bypassing the pin_read method of teensycore because it's too slow.
 */
pub fn fdd_read_index() -> u32 {
    return read_word(addrs::GPIO9) & (0x1 << 5);
}

/**
 * True if the media is write protected
 */
pub fn fdd_read_write_protect() -> bool {
    return pin_read(WRITE_PROTECT_PIN) > 0;
}

/**
 * True if the device is oriented on track0
 */
fn fdd_sense_track00() -> bool {
    return pin_read(TRACK00_PIN) == 0;
}

/**
 * Make the drive inactive
 */
fn fdd_drive_deselect() {
    pin_out(DRIVE_PIN, Power::High);
    wait_exact_ns(MS_TO_NANO * 500);
}

/** Make the drive active */
fn fdd_drive_select() {
    fdd_drive_deselect();

    pin_out(DRIVE_PIN, Power::Low);
    wait_exact_ns(MS_TO_NANO * 500);
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
        pull_keep: PullKeep::Keeper,
        pull_keep_en: false,
        open_drain: false,
        speed: PinSpeed::Max200MHz,
        drive_strength: DriveStrength::Max,
        fast_slew_rate: true,
    };

    pin_pad_config(GATE_PIN, generic_config.clone());
    pin_pad_config(DIR_PIN, generic_config.clone());
    pin_pad_config(STEP_PIN, generic_config.clone());
    pin_pad_config(HEAD_SEL_PIN, generic_config.clone());
    pin_pad_config(DRIVE_PIN, generic_config.clone());
    pin_pad_config(MOTOR_PIN, generic_config.clone());
    pin_pad_config(WRITE_PIN, generic_config.clone());

    pin_out(DRIVE_PIN, Power::High);
    pin_out(MOTOR_PIN, Power::High);
    pin_out(DIR_PIN, Power::High);
    pin_out(STEP_PIN, Power::High);
    pin_out(HEAD_SEL_PIN, Power::High);
    pin_out(GATE_PIN, Power::High);
    pin_out(WRITE_PIN, Power::High);

    pin_mode(DIR_PIN, Mode::Output);
    pin_mode(STEP_PIN, Mode::Output);
    pin_mode(GATE_PIN, Mode::Output);
    pin_mode(HEAD_SEL_PIN, Mode::Output);
    pin_mode(WRITE_PIN, Mode::Output);
    pin_mode(DRIVE_PIN, Mode::Output);
    pin_mode(MOTOR_PIN, Mode::Output);

    // Create a generic configuration for pullup resistors
    let pullup_config: PadConfig = PadConfig {
        hysterisis: false,
        resistance: PullUpDown::PullUp47k,
        pull_keep: PullKeep::Pull,
        pull_keep_en: true,
        open_drain: true,
        speed: PinSpeed::Max200MHz,
        drive_strength: DriveStrength::Max,
        fast_slew_rate: true,
    };

    // Set them to outputs
    pin_mode(INDEX_PIN, Mode::Input);
    pin_mode(TRACK00_PIN, Mode::Input);
    pin_mode(WRITE_PROTECT_PIN, Mode::Input);
    pin_mode(READY_PIN, Mode::Input);
    pin_mode(READ_PIN, Mode::Input);
    pin_pad_config(INDEX_PIN, pullup_config.clone());
    pin_pad_config(TRACK00_PIN, pullup_config.clone());
    pin_pad_config(WRITE_PROTECT_PIN, pullup_config.clone());
    pin_pad_config(READY_PIN, pullup_config.clone());
    pin_pad_config(READ_PIN, pullup_config.clone());
}

/**
 * Change the state of the motor.
 */
pub fn fdd_set_motor(on: bool) {
    let motor_active = unsafe { FLOPPY_MOTOR_ON };

    // If the motor is unchanged, don't do anything
    if on == motor_active {
        return;
    }

    if on {
        // Turn on the motor
        pin_out(MOTOR_PIN, Power::Low);
        // Select the drive
        fdd_drive_select();
        // Seek to 0
        match fdd_seek_track00() {
            None => {
                debug_str(b"Failed power-on calibration");
            }
            Some(_) => {
                debug_str(b"Successfully calibrated track");
            }
        }
    } else {
        fdd_drive_deselect();
        pin_out(MOTOR_PIN, Power::High);
    }

    if !on {
        debug_str(b"Shutting down motor");

        unsafe {
            FLOPPY_MOTOR_ON = false;
        }
        return;
    }

    debug_str(b"Spinning up motor");
    debug_str(b"Waiting for index pulse...");

    // Do a step

    let start = nanos();
    while fdd_read_index() > 0 && (nanos() - start) < 10000 * MS_TO_NANO {
        assembly!("nop");
    }

    if fdd_read_index() == 0 {
        debug_str(b"Received index pulse!");
        unsafe {
            FLOPPY_MOTOR_ON = true;
        }
    } else {
        debug_str(b"Did not receive index pulse");
        pin_out(MOTOR_PIN, Power::High);
    }
}

/**
 * Change the active track.
 */
pub fn fdd_step(times: u8) {
    for _ in 0..times {
        pin_out(STEP_PIN, Power::Low);
        wait_exact_ns(MS_TO_NANO * 3);
        pin_out(STEP_PIN, Power::High);
        wait_exact_ns(MS_TO_NANO * 3);
    }
}

fn fdd_step_dir(dir: Power) {
    pin_out(DIR_PIN, dir);
    wait_exact_ns(20 * MS_TO_NANO);
}

/**
 * Seek to track 0.
 */
pub fn fdd_seek_track00() -> Option<usize> {
    let mut cycles: usize = 0;

    debug_str(b"Seeking outwards...");
    fdd_step_dir(Power::High);
    for _ in 0..120 {
        if fdd_sense_track00() {
            unsafe {
                FLOPPY_TRACK = 0;
            }
            wait_exact_ns(MS_TO_NANO * 20);
            return Some(cycles);
        }

        cycles += 1;
        fdd_step(1);
    }

    debug_str(b"Seeking inwards...");
    fdd_step_dir(Power::Low);
    for _ in 0..20 {
        if fdd_sense_track00() {
            unsafe {
                FLOPPY_TRACK = 0;
            }
            wait_exact_ns(MS_TO_NANO * 20);
            return Some(cycles);
        }

        cycles += 1;
        fdd_step(1);
    }

    return None;
}

/**
 * Navigate to a specific track
 */
fn fdd_set_track(track: u8) {
    let cur = unsafe { FLOPPY_TRACK };
    if cur == track {
        return;
    } else if cur > track {
        // Step right
        fdd_step_dir(Power::High);
        fdd_step(cur - track);
    } else {
        // Step left
        fdd_step_dir(Power::Low);
        fdd_step(track - cur);
    }

    unsafe {
        FLOPPY_TRACK = track;
    }
}

fn fdd_fix_track(desired_track: u8, sampled_track: u8) {
    unsafe {
        FLOPPY_TRACK = sampled_track;
    }

    fdd_set_track(desired_track);
}

fn fdd_set_side(side: u8) {
    if side == 0 {
        pin_out(HEAD_SEL_PIN, Power::High);
    } else {
        pin_out(HEAD_SEL_PIN, Power::Low);
    }
}

/**
 * Read an entire sector
 */
pub fn fdd_read_sector(head: u8, cylinder: u8, sector: u8) -> Option<SectorID> {
    fdd_set_track(cylinder);
    fdd_set_side(head);

    let mut error = 0usize;
    let mut buf: [u8; 560] = [0; 560];
    let mut ret = SectorID::new();
    let offset = 45;
    while error < 36 {
        if (mfm_sync()) {
            mfm_read_bytes(&mut buf);

            // If we're on the wrong track, shimmy over to the correct one
            if buf[0] == 0xFE && buf[1] != cylinder {
                fdd_fix_track(cylinder, buf[1] as u8);
            } else if buf[0] == 0xFE && buf[1] == cylinder && buf[2] == head && buf[3] == sector {
                ret.id = buf[0];
                ret.cylinder = buf[1];
                ret.head = buf[2];
                ret.sector = buf[3];
                ret.size = buf[4];

                // Copy the data
                for i in 0..512 {
                    ret.data[i] = buf[i + offset];
                }

                // TODO:  crc stuff
                return Some(ret);
            }
        }

        if fdd_read_index() == 0 {
            while fdd_read_index() == 0 {
                assembly!("nop");
            }
            error += 1;
        }
    }

    return None;
}

pub fn fdd_write_sector(head: u8, cylinder: u8, sector: u8, data: &[u8]) -> bool {
    // The algorithm will work like so:
    // First, seek the sector we want and then read the first 60 bytes
    // which are the metadata. Compare with target. If approved then
    // write based on timing.
    fdd_set_side(head);
    fdd_set_track(cylinder);
    let mut error = 0usize;
    let mut buf: [u8; 15] = [0; 15];
    let mut byte_buf: [u8; 1] = [0; 1];
    let mut flux_signals: [Symbol; 4096] = [Symbol::Pulse10; 4096];

    // Prepare the data
    let signal_count = mfm_prepare_write(data, &mut flux_signals);
    let mut latch = false;

    while error < 10 {
        if (mfm_sync()) {
            mfm_read_bytes(&mut buf);

            // If we're on the wrong track, shimmy over to the correct one
            if buf[0] == 0xFE && buf[1] != cylinder {
                fdd_fix_track(cylinder, buf[1] as u8);
            } else if buf[0] == 0xFE && buf[1] == cylinder && buf[2] == head && buf[3] == sector {
                mfm_sync();
                mfm_read_bytes(&mut byte_buf);

                if byte_buf[0] == 0xFB || byte_buf[0] == 0xFA {
                    // Write the data
                    // debug_str(b"Found sector");
                    mfm_write_bytes(&flux_signals[0..signal_count]);
                    return true;
                } else {
                    debug_str(b"Failed to synchronize the bytes");
                    debug_u64(byte_buf[0] as u64, b"byte_buf[0]");
                    return false;
                }
            }
        }

        if fdd_read_index() == 0 {
            if latch == false {
                error += 1;
            }
            latch = true;
        } else {
            latch = false;
        }
    }

    return false;
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
