use core::arch::asm;
use core::arch::global_asm;
use teensycore::phys::addrs;
use teensycore::prelude::*;

use crate::config::WRITE_PIN;
use crate::mfm;

global_asm!(include_str!("timing.S"));

extern "C" {
    #[link_section = ".text.main"]
    #[inline(never)]
    pub fn pulse(cycles: u32);

    #[link_section = ".text.main"]
    pub fn wait_cycle(cycles: u32);

    #[link_section = ".text.main"]
    #[inline(never)]
    pub fn read_sym() -> i16;
}

#[no_mangle]
#[link_section = ".text.main"]
fn data_low() {
    assign(addrs::GPIO7 + 0x88, 0x1 << 16);
}

#[no_mangle]
#[link_section = ".text.main"]
fn data_high() {
    assign(addrs::GPIO7 + 0x84, 0x1 << 16);
}
