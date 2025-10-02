extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use bitvec::prelude::*;

use crate::epd_driver::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackError {
    BadInputLen { expected: usize, got: usize },
    OutBufTooSmall { needed: usize, got: usize },
    Overflow,
}

#[inline]
const fn round_up_to_8(n: usize) -> usize {
    (n + 7) & !7
}

/// Packing: row-major, MSB-first per byte; tail bits → forced white.
pub fn pack_bitmap<T: BitStore>(
    bits: &BitSlice<T, Msb0>, // len == w*h
    x: usize,
    y: usize,
    w: usize,
    h: usize,
) -> Result<(Rect, Box<[u8]>), PackError> {
    if w == 0 || h == 0 {
        let rect = Rect {
            x: x & !7,
            y,
            w: 0,
            h: 0,
        };
        return Ok((rect, Box::from(&[][..])));
    }

    let expected_bits = w.checked_mul(h).ok_or(PackError::Overflow)?;
    if bits.len() != expected_bits {
        return Err(PackError::BadInputLen {
            expected: expected_bits,
            got: bits.len(),
        });
    }

    let shift = x & 7; // left-padding (white) bits each row
    let x_aligned = x & !7;
    let w_aligned = round_up_to_8(w + shift);
    let line_bytes = w_aligned / 8;

    let total_bytes = line_bytes.checked_mul(h).ok_or(PackError::Overflow)?;
    let mut buf: Vec<u8> = vec![0u8; total_bytes];

    for row in 0..h {
        // Source slice for this row: exactly `w` bits
        let row_bits = &bits[row * w..row * w + w];
        let line = &mut buf[row * line_bytes..(row + 1) * line_bytes];
        // Clear line (already zeroed by vec![0; ...], but keep explicit for clarity)
        // for b in line.iter_mut() { *b = 0; }

        // Place each source bit at destination position `shift + k`
        // MSB-first: within a byte, bit 7 is the leftmost pixel.
        for (k, bit) in row_bits.iter().by_vals().enumerate() {
            if !bit {
                continue;
            }
            let pos = shift + k; // bit index in the destination scanline
            let byte_ix = pos / 8;
            let bit_in_byte = 7 - (pos % 8); // MSB-first
            // Safety: byte_ix < line_bytes by construction of w_aligned_bits
            line[byte_ix] |= 1u8 << bit_in_byte;
        }
    }

    let rect = Rect {
        x: x_aligned,
        y,
        w: w_aligned,
        h,
    };

    Ok((rect, buf.into_boxed_slice()))
}
