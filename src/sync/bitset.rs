use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const WORD_SIZE: usize = u64::BITS as usize;
const WORD_SIZE_MASK: usize = WORD_SIZE - 1;
const WORD_SIZE_SHIFT: usize = WORD_SIZE.trailing_zeros() as usize;

pub struct AtomicBitset {
    words: Vec<AtomicU64>,
    len: usize,
}

impl AtomicBitset {
    #[inline]
    pub fn with_len(len: usize) -> AtomicBitset {
        // Round up to nearest multiple of word size
        let bits_len = (len + WORD_SIZE - 1) >> WORD_SIZE_SHIFT;

        let mut words = Vec::new();
        words.resize_with(bits_len, || AtomicU64::new(0));

        AtomicBitset { words, len }
    }

    #[inline]
    pub fn set(&self, index: usize, ordering: Ordering) {
        assert!(index < self.len);

        let mask = 1 << (index & WORD_SIZE_MASK);
        self.words[index >> WORD_SIZE_SHIFT].fetch_or(mask, ordering);
    }

    #[inline]
    pub fn get(&self, index: usize, ordering: Ordering) -> bool {
        assert!(index < self.len);

        let mask = 1 << (index & WORD_SIZE_MASK);
        let word = self.words[index >> WORD_SIZE_SHIFT].load(ordering);
        word & mask != 0
    }

    #[inline]
    pub fn drain(&self, ordering: Ordering) -> Drain {
        let mut iter = self.words.iter();
        let current_word = iter.next().map(|word| word.swap(0, ordering));

        Drain {
            iter,
            ordering,
            len: self.len,
            index: 0,
            current_word,
        }
    }
}

pub struct Drain<'a> {
    iter: slice::Iter<'a, AtomicU64>,
    ordering: Ordering,
    len: usize,
    index: usize,
    current_word: Option<u64>,
}

impl<'a> Iterator for Drain<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(word) = self.current_word {
            let bit_index = word.trailing_zeros() as usize;
            if bit_index < WORD_SIZE && self.index + bit_index < self.len {
                // Zero out the bit we found
                let mask = 1 << bit_index;
                self.current_word = Some(word & !mask);

                return Some(self.index + bit_index);
            }

            self.current_word = self.iter.next().map(|word| word.swap(0, self.ordering));
            self.index += WORD_SIZE;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get() {
        let bitset = AtomicBitset::with_len(8);

        assert!(!bitset.get(0, Ordering::Relaxed));
        bitset.set(0, Ordering::Relaxed);
        assert!(bitset.get(0, Ordering::Relaxed));

        assert!(!bitset.get(3, Ordering::Relaxed));
        bitset.set(3, Ordering::Relaxed);
        assert!(bitset.get(3, Ordering::Relaxed));

        assert!(!bitset.get(7, Ordering::Relaxed));
        bitset.set(7, Ordering::Relaxed);
        assert!(bitset.get(7, Ordering::Relaxed));
    }

    #[test]
    fn drain() {
        let bitset = AtomicBitset::with_len(8);

        bitset.set(0, Ordering::Relaxed);
        bitset.set(3, Ordering::Relaxed);
        bitset.set(7, Ordering::Relaxed);

        let mut iter = bitset.drain(Ordering::Relaxed);
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
        for _ in bitset.drain(Ordering::Relaxed) {
            count += 1;
        }

        assert_eq!(count, 128);
    }
}
