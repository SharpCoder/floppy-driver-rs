use crate::config::{INDEX_PIN, READ_PIN};
use crate::fdd::fdd_read_index;
use core::arch::asm;
use teensycore::clock::F_CPU;
use teensycore::prelude::*;

const T2_5: u32 = (F_CPU * 5) / 2 / 1000000;
const T3_5: u32 = (F_CPU * 7) / 2 / 1000000;

#[no_mangle]
fn read_data() -> u32 {
    return read_word(teensycore::phys::addrs::GPIO7) & (0x1 << 1);
}

#[derive(Copy, Clone)]
enum Symbol {
    Pulse10 = 0,
    Pulse100 = 1,
    Pulse1000 = 2,
}

impl Symbol {
    fn is(&self, other: &Symbol) -> bool {
        return *self as usize == *other as usize;
    }
}

const SYNC_PATTERN: [Symbol; 15] = [
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

#[no_mangle]
fn mfm_read_sym() -> Symbol {
    let mut pulses: u32 = 5;

    while read_data() == 0 {
        pulses += 4;
    }

    while read_data() > 0 {
        pulses += 4;
    }

    if pulses < T2_5 {
        return Symbol::Pulse10;
    } else if pulses > T3_5 {
        return Symbol::Pulse1000;
    } else {
        return Symbol::Pulse100;
    }
}

pub fn mfm_sync() {
    let mut short = 0;

    loop {
        let sym = mfm_read_sym();
        if sym.is(&Symbol::Pulse10) {
            short += 1;
        } else if short > 90 {
            for sym in SYNC_PATTERN {
                if !mfm_read_sym().is(&sym) {
                    short = 0;
                    continue;
                }
            }

            // Let's eat some number of bytes
            for _ in 0..5000 {
                mfm_read_sym();
                if fdd_read_index() == 0 {
                    break;
                }
            }

            return;
        } else {
            short = 0;
        }
    }
}

pub fn mfm_read_byte() -> u8 {
    return 0;
}
