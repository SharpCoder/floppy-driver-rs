use core::arch::global_asm;
use teensycore::phys::addrs;
use teensycore::prelude::*;

#[cfg(not(feature = "testing"))]
global_asm!(include_str!("timing.S"));

extern "C" {
    pub fn pulse(cycles: u32);
    pub fn read_sym() -> i16;
    pub fn mfm_sync() -> bool;
}

#[no_mangle]
fn data_low() {
    assign(addrs::GPIO7 + 0x88, 0x1 << 16);
}

#[no_mangle]
fn data_high() {
    assign(addrs::GPIO7 + 0x84, 0x1 << 16);
}
