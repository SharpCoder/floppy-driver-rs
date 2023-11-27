use core::arch::asm;
use core::arch::global_asm;
use teensycore::phys::addrs;
use teensycore::prelude::*;

global_asm!(include_str!("timing.S"));

extern "C" {
    #[link_section = ".text.main"]
    #[inline(never)]
    pub fn pulse_10();
    #[link_section = ".text.main"]
    pub fn pulse_100();
    #[link_section = ".text.main"]
    pub fn pulse_1000();
}

#[no_mangle]
#[link_section = ".text.main"]
fn data_low() {
    assign(addrs::GPIO7 + 0x88, 0x1 << 11);
}

#[no_mangle]
#[link_section = ".text.main"]
fn data_high() {
    assign(addrs::GPIO7 + 0x84, 0x1 << 11);
}
