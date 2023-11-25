use crate::fdd::fdd_read_index;
use core::arch::asm;
use teensycore::clock::F_CPU;
use teensycore::prelude::*;

const T2_5: u32 = (F_CPU * 5) / 2 / 1000000;
const T3_5: u32 = (F_CPU * 7) / 2 / 1000000;

/**
 * This is a total hack. Read directly from the gpio register for pin 12.
 * Need to bypass the normal pin_read method in teensycore because that
 * thing is too bloated.
 */
fn read_data() -> u32 {
    return read_word(teensycore::phys::addrs::GPIO7) & (0x1 << 1);
}

#[derive(Copy, Clone)]
enum Parity {
    Even = 0xFFFF,
    Odd = 0x0,
}

impl Parity {
    pub fn as_mask(&self) -> u16 {
        return *self as u16;
    }

    pub fn is(&self, other: &Parity) -> bool {
        return *self as u16 == *other as u16;
    }

    pub fn flip(&mut self) -> Parity {
        match self {
            Parity::Even => {
                return Parity::Odd;
            }
            Parity::Odd => {
                return Parity::Even;
            }
        }
    }
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

static SYNC_PATTERN: [Symbol; 15] = [
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

/**
 * Read a flux transition and time it to one of the 3 known pulse types.
 */
fn mfm_read_sym() -> Symbol {
    let mut pulses: u32 = 5;

    while read_data() == 0 {
        pulses += 5;
    }

    while read_data() > 0 {
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

/**
 * Wait for a synchronization byte marker
 */
pub fn mfm_sync() -> bool {
    let mut short = 0;
    let mut index = 0;

    while fdd_read_index() != 0 {
        let sym = mfm_read_sym();
        if sym.is(&Symbol::Pulse10) {
            short += 1;
        } else if short > 90 && sym.is(&SYNC_PATTERN[0]) {
            let mut found = true;
            for i in 1..SYNC_PATTERN.len() {
                if !mfm_read_sym().is(&SYNC_PATTERN[i]) {
                    found = false;
                    break;
                }
            }

            if !found {
                short = 0;
                continue;
            } else {
                return true;
            }
        } else {
            short = 0;
        }
    }
    return false;
}

/**
 * Fill the array with bytes derived from the flux transitions.
 */
pub fn mfm_read_bytes(arr: &mut [u8]) -> bool {
    let mut byte: u16 = 0;
    let mut state = Parity::Even;
    let mut weight = 0x8000;
    let mut n = 0;

    // This relies on the assumption that we're hot off the press from
    // a sync marker. As such, the next flux transition has some
    // weird rules to get back into lock-step with the data bit.
    match mfm_read_sym() {
        Symbol::Pulse100 => {
            state = Parity::Odd;
            weight >>= 1;
        }
        Symbol::Pulse1000 => {
            weight >>= 1;
        }
        Symbol::Pulse10 => {}
    }

    // Read the remainder of the data.
    loop {
        // Set bit
        byte |= weight & state.as_mask();
        weight >>= 1;

        match mfm_read_sym() {
            Symbol::Pulse1000 => {
                // Since it's 3 zeros, doesn't matter what the parity is
                // next bit is guaranteed to be a zero.
                weight >>= 1;
            }
            Symbol::Pulse100 => {
                if state.is(&Parity::Even) {
                    weight >>= 1;
                }
                // For 1000 and 10 the parity remains unchanged but
                // for 100 it's an odd numbered signal so we must
                // flip the parity.
                state = state.flip();
            }
            _ => {}
        }

        // When we've exhausted the length of a byte,
        // we can write it and adjust values for the
        // follow up.
        if weight <= 0x80 {
            arr[n] = (byte >> 8) as u8;
            byte <<= 8;
            weight <<= 8;
            n += 1;

            if n == arr.len() {
                break;
            }
        }

        if fdd_read_index() == 0 {
            return false;
        }
    }

    return true;
}
