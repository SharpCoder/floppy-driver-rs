#![allow(unused)]

use core::arch::asm;
use teensycore::{phys::gpio::gpio_read_12, prelude::*};

const T2_5: u32 = (F_CPU * 5) / 2 / 1000000;
const T3_5: u32 = (F_CPU * 7) / 2 / 1000000;

#[derive(Copy, Clone)]
enum Symbol {
    Pulse10 = 0,
    Pulse100 = 1,
    Pulse1000 = 2,
}

impl Symbol {
    pub fn is(&self, other: &Symbol) -> bool {
        return *self as usize == *other as usize;
    }
}

#[derive(Clone, Copy)]
pub struct FloppyConfiguration {
    pub index_pin: usize,
    pub drive_pin: usize,
    pub motor_pin: usize,
    pub dir_pin: usize,
    pub step_pin: usize,
    pub write_pin: usize,
    pub gate_pin: usize,
    pub track00_pin: usize,
    pub write_protect_pin: usize,
    pub read_pin: usize,
    pub head_sel_pin: usize,
    pub disk_change_pin: usize,
}

#[derive(Clone, Copy)]
pub struct FloppyDriver {
    debug: bool,
    motor_active: bool,
    index_pin: usize,
    drive_pin: usize,
    motor_pin: usize,
    dir_pin: usize,
    step_pin: usize,
    write_pin: usize,
    gate_pin: usize,
    track00_pin: usize,
    write_protect_pin: usize,
    read_pin: usize,
    head_sel_pin: usize,
    disk_change_pin: usize,
}

impl FloppyDriver {
    pub fn new(config: FloppyConfiguration) -> Self {
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

        pin_pad_config(config.drive_pin, generic_config.clone());
        pin_pad_config(config.motor_pin, generic_config.clone());
        pin_pad_config(config.dir_pin, generic_config.clone());
        pin_pad_config(config.step_pin, generic_config.clone());
        pin_pad_config(config.write_pin, generic_config.clone());
        pin_pad_config(config.gate_pin, generic_config.clone());
        pin_pad_config(config.head_sel_pin, generic_config.clone());

        pin_mode(config.drive_pin, Mode::Output);
        pin_mode(config.motor_pin, Mode::Output);
        pin_mode(config.dir_pin, Mode::Output);
        pin_mode(config.step_pin, Mode::Output);
        pin_mode(config.head_sel_pin, Mode::Output);
        pin_mode(config.write_pin, Mode::Output);
        pin_mode(config.gate_pin, Mode::Output);

        pin_out(config.drive_pin, Power::High);
        pin_out(config.motor_pin, Power::High);
        pin_out(config.dir_pin, Power::High);
        pin_out(config.step_pin, Power::High);
        pin_out(config.head_sel_pin, Power::High);
        pin_out(config.write_pin, Power::High);
        pin_out(config.gate_pin, Power::High);

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

        pin_pad_config(config.index_pin, pullup_config.clone());
        pin_pad_config(config.track00_pin, pullup_config.clone());
        pin_pad_config(config.write_protect_pin, pullup_config.clone());
        pin_pad_config(config.disk_change_pin, pullup_config.clone());

        // Read pin specifically
        pin_pad_config(
            config.read_pin,
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
        pin_mode(config.index_pin, Mode::Input);
        pin_mode(config.track00_pin, Mode::Input);
        pin_mode(config.write_protect_pin, Mode::Input);
        pin_mode(config.read_pin, Mode::Input);
        pin_mode(config.disk_change_pin, Mode::Input);

        return FloppyDriver {
            debug: true,
            motor_active: false,
            index_pin: config.index_pin,
            drive_pin: config.drive_pin,
            motor_pin: config.motor_pin,
            dir_pin: config.dir_pin,
            step_pin: config.step_pin,
            write_pin: config.write_pin,
            gate_pin: config.gate_pin,
            track00_pin: config.track00_pin,
            write_protect_pin: config.write_protect_pin,
            read_pin: config.read_pin,
            head_sel_pin: config.head_sel_pin,
            disk_change_pin: config.disk_change_pin,
        };
    }

    fn soft_reset(&mut self) {
        pin_out(self.drive_pin, Power::High);
        pin_out(self.motor_pin, Power::High);
        pin_out(self.dir_pin, Power::High);
        pin_out(self.step_pin, Power::High);
        pin_out(self.write_pin, Power::High);
        pin_out(self.gate_pin, Power::High);
        pin_out(self.head_sel_pin, Power::High);

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
            pin_out(self.gate_pin, Power::High);
            pin_out(self.drive_pin, Power::High);
            pin_out(self.head_sel_pin, Power::High);
            pin_out(self.motor_pin, Power::High);
            wait_exact_ns(MS_TO_NANO * 3000);
            pin_out(self.drive_pin, Power::Low);
            pin_out(self.head_sel_pin, Power::High);
            pin_out(self.motor_pin, Power::Low);
            wait_exact_ns(MS_TO_NANO * 1000);
        } else {
            pin_out(self.motor_pin, Power::High);
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
        while pin_read(self.index_pin) > 0 && (nanos() - start) < 10000 * MS_TO_NANO {
            assembly!("nop");
        }

        if pin_read(self.index_pin) == 0 {
            debug_str(b"Received index pulse!");
            wait_exact_ns(MS_TO_NANO * 5000);
        } else {
            debug_str(b"Did not receive index pulse");
            self.motor_active = false;
        }
    }

    pub fn step(&self, dir: Power, times: u8) {
        pin_out(self.dir_pin, dir);
        for _ in 0..times {
            pin_out(self.step_pin, Power::High);
            wait_exact_ns(MS_TO_NANO * 11);
            pin_out(self.step_pin, Power::Low);
            wait_exact_ns(MS_TO_NANO * 11);
            pin_out(self.step_pin, Power::High);
        }
    }

    #[no_mangle]
    fn read_data(&self) -> bool {
        return gpio_read_12() > 0;
    }

    #[no_mangle]
    fn read_sym(&self) -> Symbol {
        let mut pulses: u32 = 5;

        while gpio_read_12() == 0 {
            pulses += 5;
        }

        while gpio_read_12() > 0 {
            pulses += 5;
        }

        if pulses < T2_5 {
            return Symbol::Pulse10;
        } else if pulses > T3_5 {
            return Symbol::Pulse1000;
        } else {
            return Symbol::Pulse100;
        }
    }

    #[no_mangle]
    pub fn read_track(&mut self) {
        // self.motor_on(true);

        let mut pulses_10 = 0;
        let mut pulses_100 = 0;
        let mut pulses_1000 = 0;
        let mut found_sync = false;
        let mut sync_error = false;
        let mut sync = 0;
        let mut sync_iterations = 0;

        // Wait for an index pulse
        while pin_read(self.index_pin) != 0 {}
        while pin_read(self.index_pin) == 0 {}

        let mut pattern_index = 0;
        // MLMLMSLMLMSLMLM
        let pattern = [
            Symbol::Pulse100,
            Symbol::Pulse1000,
            Symbol::Pulse100,
            Symbol::Pulse1000,
            Symbol::Pulse100,
            Symbol::Pulse10,
            Symbol::Pulse1000,
            Symbol::Pulse100,
            Symbol::Pulse1000,
            Symbol::Pulse100,
            Symbol::Pulse10,
            Symbol::Pulse1000,
            Symbol::Pulse100,
            Symbol::Pulse1000,
            Symbol::Pulse100,
        ];

        let start = nanos() / MS_TO_NANO;
        while pin_read(self.index_pin) != 0 {
            if pin_read(self.read_pin) == 0 {
                let mut sym = self.read_sym();

                match sym {
                    Symbol::Pulse100 => {
                        pulses_100 += 1;
                    }

                    Symbol::Pulse1000 => {
                        pulses_1000 += 1;
                    }

                    Symbol::Pulse10 => {
                        pulses_10 += 1;
                    }
                }

                if sym.is(&Symbol::Pulse10) && pattern_index == 0 {
                    sync += 1;
                } else if sync >= 80 && sym.is(&pattern[pattern_index]) {
                    if pattern_index < 14 {
                        pattern_index += 1;
                    } else {
                        found_sync = true;
                        sync_iterations += 1;
                        sync = 0;
                        pattern_index = 0;
                        pulses_10 = 0;
                        pulses_100 = 0;
                        pulses_1000 = 0;
                    }
                } else {
                    sync = 0;
                    pattern_index = 0;
                }
            }
        }

        let end = nanos() / MS_TO_NANO;

        // debug_u64(pulses_10 + pulses_100 + pulses_1000, b"PULSES");

        debug_u64(pulses_10 as u64, b"PULSES_10");
        debug_u64(pulses_100 as u64, b"PULSES_100");
        debug_u64(pulses_1000 as u64, b"PULSES_1000");
        debug_u64(
            pulses_10 + pulses_100 + pulses_1000 as u64,
            b"Total Flux Transitions in one sector",
        );

        match found_sync {
            true => {
                debug_str(b"Found sync pattern");
            }
            false => {
                debug_str(b"Did not find sync pattern");
            }
        }

        debug_u64(sync_iterations, b"Sectors found");

        // debug_u64(pulses_100 as u64, b"PULSES_100");
        // debug_u64(pulses_1000 as u64, b"PULSES_1000");
        // debug_u64((pulses_1000 + pulses_100) as u64, b"Total Pulses");
        // debug_u64((end - start) as u64, b"TIMING\n");
    }

    pub fn begin(&mut self) {
        self.soft_reset();
        self.motor_on(true);
    }

    pub fn seek_track00(&mut self) -> Option<usize> {
        self.motor_on(true);
        let mut cycles: usize = 0;

        for _ in 0..100 {
            if pin_read(self.track00_pin) == 0 {
                return Some(cycles);
            }

            cycles += 1;
            self.step(Power::High, 1);
        }

        for _ in 0..20 {
            if pin_read(self.track00_pin) == 0 {
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
