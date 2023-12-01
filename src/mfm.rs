use crate::fdd::fdd_read_index;
use core::arch::asm;
use core::arch::global_asm;
use teensycore::prelude::*;

#[cfg(not(feature = "testing"))]
global_asm!(include_str!("mfm.S"));

extern "C" {
    pub fn _asm_pulse(cycles: u32);
    pub fn _asm_read_sym() -> i16;
    pub fn _asm_sync() -> bool;
    pub fn _asm_full_write_test();
}

// const CYCLES_PER_MICRO: u32 = F_CPU / 1000000;
// const CLOCK_PER_MICRO: u32 = CLOCK_CPU / 1000000;

// const T: u32 = CYCLES_PER_MICRO * 5 / 3;
const T2: u32 = 544 * 2 / 3; //1.375 * CYCLES_PER_MICRO;
const T3: u32 = 940 / 2; //2.375 * CYCLES_PER_MICRO;
const T4: u32 = 1336 * 2 / 3; //3.375 * CYCLES_PER_MICRO;

/**
This is a total hack. Read directly from the gpio register for pin 12.
 Need to bypass the normal pin_read method in teensycore because that
 thing is too bloated.
*/
#[no_mangle]
#[inline(never)]
fn read_data() -> u32 {
    unsafe {
        return *(addrs::GPIO7 as *mut u32) & (0x1 << 1);
    }
}

#[no_mangle]
#[inline(never)]
#[link_section = ".text"]
fn open_gate() {
    unsafe {
        *((addrs::GPIO7 + 0x88) as *mut u32) = 0x1 << 11;
    }
}

#[no_mangle]
#[inline(never)]
#[link_section = ".text"]
fn close_gate() {
    unsafe {
        *((addrs::GPIO7 + 0x84) as *mut u32) = 0x1 << 11;
    }
}

#[no_mangle]
#[link_section = ".text"]
#[inline(never)]
pub fn data_low() {
    unsafe {
        *((addrs::GPIO7 + 0x88) as *mut u32) = 0x1 << 16;
    }
}

#[no_mangle]
#[link_section = ".text"]
#[inline(never)]
pub fn data_high() {
    unsafe {
        *((addrs::GPIO7 + 0x84) as *mut u32) = 0x1 << 16;
    }
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
    fn from(count: i16) -> Self {
        return match count {
            0 => Self::Pulse10,
            1 => Self::Pulse100,
            _ => Self::Pulse1000,
        };
    }
}

fn simplify(byte: u16) -> u16 {
    if byte > 0 {
        return 1;
    } else {
        return 0;
    }
}

#[cfg(not(testing))]
#[inline(never)]
pub fn mfm_read_sym() -> Symbol {
    return unsafe { Symbol::from(_asm_read_sym()) };
}

/**
This method will dump the bucketed counts of symbols across
one index loop
 */
pub fn mfm_dump_stats() {
    while fdd_read_index() != 0 {
        assembly!("nop");
    }

    while fdd_read_index() == 0 {
        assembly!("nop");
    }

    let mut pulse_10 = 0;
    let mut pulse_100 = 0;
    let mut pulse_1000 = 0;

    while fdd_read_index() != 0 {
        match mfm_read_sym() {
            Symbol::Pulse10 => {
                pulse_10 += 1;
            }
            Symbol::Pulse100 => {
                pulse_100 += 1;
            }
            Symbol::Pulse1000 => {
                pulse_1000 += 1;
            }
        }
    }

    debug_u64(pulse_10 as u64, b"pulse_10");
    debug_u64(pulse_100 as u64, b"pulse_100");
    debug_u64(pulse_1000 as u64, b"pulse_1000");
}

#[no_mangle]
#[inline(never)]
#[link_section = ".text"]
pub fn mfm_read_flux(dst: &mut [Symbol; 4096], len: usize) {
    for i in 0..len {
        dst[i] = unsafe { Symbol::from(_asm_read_sym()) };
    }
}

/**
 * Wait for a synchronization byte marker
 */
#[cfg(not(testing))]
pub fn mfm_sync() -> bool {
    unsafe {
        return _asm_sync();
    }
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

pub fn mfm_prepare_write(
    prefix_byte: u8,
    bytes: &[u8],
    flux_signals: &mut [Symbol; 4096],
) -> usize {
    let mut signal_index = 0;
    let mut ind = 0;
    let mut byte = prefix_byte as u16; // The first byte after a data barrier must be a 0xFB or 0xFA
    let mut next = bytes[0] as u16;
    let mut sigmask = 0x8000;
    let mut bitmask: u16 = 0x80;
    let mut x: u16 = byte & bitmask;
    let mut sym = -1;

    loop {
        sigmask >>= 1;
        bitmask >>= 1;

        // Process symbols
        if x > 0 && sym >= 0 {
            // Process it
            flux_signals[signal_index] = Symbol::from(sym);
            signal_index += 1;
            sym = -1;
        } else if x > 0 {
            sym = -1;
        } else {
            sym += 1;
        }

        let y = match bitmask > 0 {
            true => byte & bitmask,
            _ => next & 0x80,
        };

        let z = !(simplify(x) | simplify(y)) & 0x1;
        if z > 0 {
            // Process symbols
            if sym >= 0 {
                // Process it
                flux_signals[signal_index] = Symbol::from(sym);
                signal_index += 1;
            }

            sym = -1;
        } else {
            sym += 1;
        }

        sigmask >>= 1;
        x = y;

        if sigmask == 0 && ind < bytes.len() {
            sigmask = 0x8000;
            bitmask = 0x80;
            byte = bytes[ind] as u16;
            ind += 1;
            next = match ind < bytes.len() {
                true => bytes[ind] as u16,
                false => 0,
            };

            x |= byte & bitmask;
        } else if sigmask == 0 {
            break;
        }
    }

    if sym >= 0 {
        // Process it
        flux_signals[signal_index] = Symbol::from(sym);
        signal_index += 1;
    }

    return signal_index;
}

#[no_mangle]
#[inline(never)]
pub fn mfm_write_bytes(flux_signals: &[Symbol]) {
    open_gate();
    for sym in flux_signals {
        unsafe {
            match sym {
                Symbol::Pulse10 => _asm_pulse(T2),
                Symbol::Pulse100 => _asm_pulse(T3),
                Symbol::Pulse1000 => _asm_pulse(T4),
            };
        }
    }
    close_gate();
    data_high();
}

// Test the encoding logic
#[cfg(test)]
mod test_mfm {
    extern crate std;

    use super::mfm_prepare_write;
    use crate::mfm::mfm_write_bytes;
    use crate::mfm::Symbol;

    use std::*;

    #[test]
    pub fn test_encoding() {
        let mut flux_signals: [Symbol; 4096] = [Symbol::Pulse10; 4096];
        let signal_counts =
            mfm_prepare_write(0xFB, &[0xF6, 0xF6, 0xF6, 0xF6, 0xF6], &mut flux_signals);

        let signals = b"SSSSLSSSSSLSLSSSLSLSSSLSLSSSLSLSSSLSM ";
        // assert_eq!(signal_counts, signals.len());

        for i in 0..signal_counts {
            println!("Evaluating signal {i}");
            let sym = signals[i];
            match signals[i] {
                b'S' => {
                    assert_eq!(flux_signals[i] as usize, 0);
                }
                b'M' => {
                    assert_eq!(flux_signals[i] as usize, 1);
                }
                _ => {
                    assert_eq!(flux_signals[i] as usize, 2);
                }
            }
        }
    }
}
