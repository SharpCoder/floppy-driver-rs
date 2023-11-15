use core::arch::asm;
use teensycore::prelude::*;

fn bool_to_power(on: bool) -> Power {
    return match on {
        true => Power::High,
        false => Power::Low,
    };
}

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
    pub fn new(
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
    ) -> Self {
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

        pin_pad_config(drive_pin, generic_config.clone());
        pin_pad_config(motor_pin, generic_config.clone());
        pin_pad_config(dir_pin, generic_config.clone());
        pin_pad_config(step_pin, generic_config.clone());
        pin_pad_config(write_pin, generic_config.clone());
        pin_pad_config(gate_pin, generic_config.clone());
        pin_pad_config(head_sel_pin, generic_config.clone());

        pin_mode(drive_pin, Mode::Output);
        pin_mode(motor_pin, Mode::Output);
        pin_mode(dir_pin, Mode::Output);
        pin_mode(step_pin, Mode::Output);
        pin_mode(head_sel_pin, Mode::Output);
        pin_mode(write_pin, Mode::Output);
        pin_mode(gate_pin, Mode::Output);

        pin_out(drive_pin, Power::High);
        pin_out(motor_pin, Power::High);
        pin_out(dir_pin, Power::High);
        pin_out(step_pin, Power::High);
        pin_out(head_sel_pin, Power::High);
        pin_out(write_pin, Power::High);
        pin_out(gate_pin, Power::High);

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

        pin_pad_config(index_pin, pullup_config.clone());
        pin_pad_config(track00_pin, pullup_config.clone());
        pin_pad_config(write_protect_pin, pullup_config.clone());
        pin_pad_config(disk_change_pin, pullup_config.clone());

        // Read pin specialness
        pin_pad_config(
            read_pin,
            PadConfig {
                hysterisis: false,
                resistance: PullUpDown::PullUp100k,
                pull_keep: PullKeep::Pull,
                pull_keep_en: true,
                open_drain: true,
                speed: PinSpeed::Fast150MHz,
                drive_strength: DriveStrength::MaxDiv3,
                fast_slew_rate: true,
            },
        );

        // Set them to outputs
        pin_mode(index_pin, Mode::Input);
        pin_mode(track00_pin, Mode::Input);
        pin_mode(write_protect_pin, Mode::Input);
        pin_mode(read_pin, Mode::Input);
        pin_mode(disk_change_pin, Mode::Input);

        return FloppyDriver {
            debug: true,
            motor_active: false,

            index_pin: index_pin,
            drive_pin: drive_pin,
            motor_pin: motor_pin,
            dir_pin: dir_pin,
            step_pin: step_pin,
            write_pin: write_pin,
            gate_pin: gate_pin,
            track00_pin: track00_pin,
            write_protect_pin: write_protect_pin,
            read_pin: read_pin,
            head_sel_pin: head_sel_pin,
            disk_change_pin: disk_change_pin,
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
            pin_out(self.gate_pin, Power::High);

            debug_str(b"Power cycling...");
            pin_out(self.drive_pin, Power::High);
            pin_out(self.motor_pin, Power::High);
            pin_out(self.head_sel_pin, Power::High);
            wait_exact_ns(MS_TO_NANO * 3000);
            pin_out(self.motor_pin, Power::Low);
            wait_exact_ns(MS_TO_NANO * 250);
            pin_out(self.drive_pin, Power::Low);
            wait_exact_ns(MS_TO_NANO * 250);
            pin_out(self.head_sel_pin, Power::High);
            wait_exact_ns(MS_TO_NANO * 250);
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

    pub fn step(&self, dir: bool, times: u8) {
        pin_out(self.dir_pin, bool_to_power(dir));
        for _ in 0..times {
            pin_out(self.step_pin, Power::High);
            wait_exact_ns(MS_TO_NANO * 11);
            pin_out(self.step_pin, Power::Low);
            wait_exact_ns(MS_TO_NANO * 11);
            pin_out(self.step_pin, Power::High);
        }
    }

    fn read_data(&self) -> bool {
        let mut signal = true;
        for _ in 0..20 {
            if pin_read(self.read_pin) == 0 {
                signal = false;
            }
        }

        return signal;
    }

    pub fn read_track(&mut self) {
        self.motor_on(true);

        let mut pulses: u32 = 0;

        // Wait for an index pulse
        while pin_read(self.index_pin) != 0 {}
        while pin_read(self.index_pin) == 0 {}

        // Begin
        let start = nanos() / MS_TO_NANO;

        while pin_read(self.index_pin) != 0 {
            while self.read_data() == true {}
            while self.read_data() == false {}
            // while pin_read(self.read_pin) != 0 {}

            pulses += 1;
        }

        let end = nanos() / MS_TO_NANO;

        debug_u64(pulses as u64, b"PULSES");
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
            if pin_read(self.track00_pin) == 0 {
                return Some(cycles);
            }

            cycles += 1;
            self.step(true, 1);
        }

        for _ in 0..20 {
            if pin_read(self.track00_pin) == 0 {
                return Some(cycles);
            }

            cycles += 1;
            self.step(false, 1);
        }

        return None;
    }

    pub fn measure_sector(&self) {}
    pub fn end(&self) {}
}
