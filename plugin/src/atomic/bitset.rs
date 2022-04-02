use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const CELL_SIZE: usize = u64::BITS as usize;

// Cells are 64 = 2^6 bits
const CELL_SIZE_BITS: usize = 6;

fn mask_for_index(index: usize) -> u64 {
    1 << (index & (CELL_SIZE - 1))
}

pub struct AtomicBitset {
    len: usize,
    bits: Vec<AtomicU64>,
}

impl AtomicBitset {
    pub fn with_len(len: usize) -> AtomicBitset {
        // Round up to nearest multiple of cell size
        let bits_len = (len + CELL_SIZE - 1) >> CELL_SIZE_BITS;
        let mut bits = Vec::with_capacity(bits_len);
        for _ in 0..bits_len {
            bits.push(AtomicU64::new(0));
        }

        AtomicBitset { len, bits }
    }

    pub fn set(&self, index: usize, ordering: Ordering) {
        assert!(index < self.len);

        self.bits[index >> CELL_SIZE_BITS].fetch_or(mask_for_index(index), ordering);
    }

    #[inline]
    pub fn drain_indices(&self, ordering: Ordering) -> DrainIndices {
        let mut iter = self.bits.iter();
        let first_cell = iter.next().map_or(0, |cell| cell.swap(0, ordering));

        DrainIndices { iter, ordering, index: 0, current_cell: first_cell }
    }
}

pub struct DrainIndices<'a> {
    iter: slice::Iter<'a, AtomicU64>,
    ordering: Ordering,
    index: usize,
    current_cell: u64,
}

impl<'a> Iterator for DrainIndices<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let bit_index = self.current_cell.trailing_zeros() as usize;
            if bit_index < CELL_SIZE {
                // Zero out the bit we found
                self.current_cell &= !mask_for_index(bit_index);
                return Some(self.index + bit_index);
            }

            if let Some(cell) = self.iter.next() {
                self.current_cell = cell.swap(0, self.ordering);
                self.index += CELL_SIZE;
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let bitset = AtomicBitset::with_len(8);

        bitset.set(0, Ordering::Relaxed);
        bitset.set(3, Ordering::Relaxed);
        bitset.set(7, Ordering::Relaxed);

        let mut iter = bitset.drain_indices(Ordering::Relaxed);
        assert_eq!(iter.next().unwrap(), 0);
        assert_eq!(iter.next().unwrap(), 3);
        assert_eq!(iter.next().unwrap(), 7);
    }

    #[test]
    fn count() {
        let bitset = AtomicBitset::with_len(1000);

        for x in 0..128 {
            bitset.set(5 + 7 * x, Ordering::Relaxed);
        }

        let mut count = 0;
        for i in bitset.drain_indices(Ordering::Relaxed) {
            println!("{}", i);
            count += 1;
        }

        assert_eq!(count, 128);
    }
}
