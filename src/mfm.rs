use crate::fdd::fdd_read_index;
use crate::timing;
use core::arch::asm;
use core::arch::global_asm;
use teensycore::clock::F_CPU;
use teensycore::prelude::*;

// some magic here to calculate how many cycles per micro
const CYCLES_PER_MICRO: u32 = F_CPU / 1000000;
const T2: u32 = 2 * CYCLES_PER_MICRO;
const T3: u32 = 3 * CYCLES_PER_MICRO;
const T4: u32 = 4 * CYCLES_PER_MICRO;
const T2_5: u32 = (F_CPU * 5) / 2 / 1000000;
const T3_5: u32 = (F_CPU * 7) / 2 / 1000000;

/**
This is a total hack. Read directly from the gpio register for pin 12.
 Need to bypass the normal pin_read method in teensycore because that
 thing is too bloated.
*/
fn read_data() -> u32 {
    let r = CYCLES_PER_MICRO;
    return read_word(teensycore::phys::addrs::GPIO7) & (0x1 << 1);
}

fn open_gate() {
    // Pull low
    assign(addrs::GPIO7 + 0x88, 0x1 << 11);
}

fn close_gate() {
    // Pull high
    assign(addrs::GPIO7 + 0x84, 0x1 << 11);
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
pub enum Symbol {
    Pulse10 = 0,
    Pulse100 = 1,
    Pulse1000 = 2,
}

impl Symbol {
    fn is(&self, other: &Symbol) -> bool {
        return *self as usize == *other as usize;
    }

    fn from(count: i16) -> Self {
        return match count {
            0 => Self::Pulse10,
            1 => Self::Pulse100,
            _ => Self::Pulse1000,
        };
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
#[no_mangle]
pub fn mfm_read_sym() -> Symbol {
    let mut pulses: u32 = 0;

    while read_data() == 0 {
        pulses += 6;
    }

    while read_data() > 0 {
        pulses += 6;
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

    while fdd_read_index() != 0 {
        let sym = mfm_read_sym();
        if sym.is(&Symbol::Pulse10) {
            short += 1;
        } else if short > 80 && sym.is(&SYNC_PATTERN[0]) {
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
        arr[n] = (byte >> 8) as u8;
        if weight <= 0x80 {
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

pub fn mfm_prepare_write(bytes: &[u8], flux_signals: &mut [Symbol; 4096]) -> usize {
    let mut signal_index = 0;

    for i in 0..bytes.len() {
        let byte = bytes[i];
        let next = match i == bytes.len() - 1 {
            true => 0,
            false => bytes[i + 1],
        };

        let signal = mfm_encode_byte(byte, next);

        // Parse the signal into symbols and record it
        let mut mask = 0x8000;
        let mut sym: i16 = -1;
        let mut begin = 0;

        if (signal & mask) == 0 {
            // We will skip the first bit
            begin = 1;
            mask >>= 1;
        };

        for _ in begin..16 {
            let bit = signal & mask;
            if bit > 0 && sym >= 0 {
                flux_signals[signal_index] = Symbol::from(sym);
                signal_index += 1;
                sym = 0;
            } else {
                sym += 1;
            }

            mask >>= 1;
        }
    }

    return signal_index;
}

#[no_mangle]
#[inline(never)]
pub fn mfm_write_bytes(flux_signals: &[Symbol]) {
    open_gate();
    for sym in flux_signals {
        unsafe {
            let target = match sym {
                Symbol::Pulse10 => timing::pulse_10(),
                Symbol::Pulse100 => timing::pulse_100(),
                Symbol::Pulse1000 => timing::pulse_1000(),
            };
        }
    }
    close_gate();
}

fn mfm_encode_byte(byte: u8, next: u8) -> u16 {
    let mut ret = 0;
    let mut mask: u16 = 0x8000;
    let mut bitmask: u16 = 0x80;
    let mut x = (byte as u16) & bitmask;

    for _ in 0..8 {
        if x > 0 {
            ret |= mask;
        }
        mask >>= 1;
        bitmask >>= 1;

        let y = match bitmask {
            0 => (next as u16) >> 7,
            _ => (byte as u16) & bitmask,
        };

        if bitmask == 0 {
            bitmask = 1;
        }

        let z = !((x >> 1) | y);

        if (z & bitmask) > 0 {
            ret |= mask;
        }

        mask >>= 1;
        x = y;
    }

    return ret;
}

// Test the encoding logic
#[cfg(test)]
mod test_mfm {
    extern crate std;

    use crate::mfm::mfm_write_bytes;

    use super::mfm_encode_byte;
    use std::*;

    #[test]
    pub fn test_interleave() {
        let byte = 0x3A;
        assert_eq!(mfm_encode_byte(byte, 0x00), 0b0100101010001001);
        assert_eq!(mfm_encode_byte(byte, 0xFF), 0b0100101010001000);

        let signal = mfm_encode_byte(0xC1, 0x7A);
        println!("{}", format!("{signal:016b}").as_str());
        assert!(false);
    }

    #[test]
    pub fn test_emit() {
        // let byte = 0x3A;
        // let signal = mfm_encode_byte(byte, 0x00);
        // // mfm_write_bytes(&[byte]);

        // println!("{}", format!("{signal:016b}").as_str());
        // assert!(false);
    }
}
