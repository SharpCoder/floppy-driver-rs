use core::arch::asm;
use core::arch::global_asm;
use teensycore::phys::addrs;
use teensycore::prelude::*;

use crate::config::WRITE_PIN;

global_asm!(include_str!("timing.S"));

extern "C" {
    #[link_section = ".text.main"]
    #[inline(never)]
    pub fn pulse_10();
    #[link_section = ".text.main"]
    #[inline(never)]
    pub fn pulse_100();
    #[link_section = ".text.main"]
    #[inline(never)]
    pub fn pulse_1000();
    #[link_section = ".text.main"]
    #[inline(never)]
    pub fn debug_wait_cycle() -> u32;
    #[link_section = ".text.main"]
    #[inline(never)]
    pub fn wait_cycle(cycles: u32) -> u32;
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
