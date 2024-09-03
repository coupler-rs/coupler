use std::sync::atomic::Ordering;

use super::bitset::{self, AtomicBitset, Bitset};
use super::float::AtomicF64;
use crate::params::ParamValue;

pub struct ParamGestures {
    values: Vec<AtomicF64>,
    gesture_states: AtomicBitset,
    dirty: AtomicBitset,
    values_dirty: AtomicBitset,
}

impl ParamGestures {
    pub fn with_count(count: usize) -> ParamGestures {
        ParamGestures {
            values: (0..count).map(|_| AtomicF64::new(0.0)).collect(),
            gesture_states: AtomicBitset::with_len(count),
            dirty: AtomicBitset::with_len(count),
            values_dirty: AtomicBitset::with_len(count),
        }
    }

    pub fn begin_gesture(&self, index: usize) {
        self.gesture_states.set(index, true, Ordering::Relaxed);
        self.dirty.set(index, true, Ordering::Release);
    }

    pub fn end_gesture(&self, index: usize) {
        self.gesture_states.set(index, false, Ordering::Relaxed);
        self.dirty.set(index, true, Ordering::Release);
    }

    pub fn set_value(&self, index: usize, value: ParamValue) {
        self.values[index].store(value, Ordering::Relaxed);
        self.values_dirty.set(index, true, Ordering::Release);
        self.dirty.set(index, true, Ordering::Release);
    }

    pub fn poll<'a, 'b>(&'a self, states: &'b mut GestureStates) -> Poll<'a, 'b> {
        Poll {
            values: &self.values,
            gesture_states: &self.gesture_states,
            values_dirty: &self.values_dirty,
            current_gestures: &mut states.states,
            iter: self.dirty.drain(Ordering::Acquire),
        }
    }
}

pub struct GestureStates {
    states: Bitset,
}

impl GestureStates {
    pub fn with_count(count: usize) -> GestureStates {
        GestureStates {
            states: Bitset::with_len(count),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GestureUpdate {
    pub index: usize,
    pub begin_gesture: bool,
    pub set_value: Option<ParamValue>,
    pub end_gesture: bool,
}

pub struct Poll<'a, 'b> {
    values: &'a [AtomicF64],
    gesture_states: &'a AtomicBitset,
    values_dirty: &'a AtomicBitset,
    current_gestures: &'b mut Bitset,
    iter: bitset::Drain<'a>,
}

impl<'a, 'b> Iterator for Poll<'a, 'b> {
    type Item = GestureUpdate;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.iter.next() {
            let mut update = GestureUpdate {
                index,
                begin_gesture: false,
                set_value: None,
                end_gesture: false,
            };

            let mut current_state = self.current_gestures.get(index);

            if self.values_dirty.swap(index, false, Ordering::Acquire) {
                if !current_state {
                    update.begin_gesture = true;
                    current_state = true;
                }

                update.set_value = Some(self.values[index].load(Ordering::Relaxed));
            }

            let next_state = self.gesture_states.get(index, Ordering::Relaxed);
            if !current_state && next_state {
                update.begin_gesture = true;
            } else if current_state && !next_state {
                update.end_gesture = true;
            }

            self.current_gestures.set(index, next_state);

            Some(update)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gesture_updates() {
        let gestures = ParamGestures::with_count(1);
        let mut states = GestureStates::with_count(1);

        let updates = gestures.poll(&mut states).collect::<Vec<_>>();
        assert!(updates.is_empty());

        gestures.begin_gesture(0);

        let updates = gestures.poll(&mut states).collect::<Vec<_>>();
        assert_eq!(
            updates,
            &[GestureUpdate {
                index: 0,
                begin_gesture: true,
                set_value: None,
                end_gesture: false,
            }]
        );

        gestures.set_value(0, 0.0);

        let updates = gestures.poll(&mut states).collect::<Vec<_>>();
        assert_eq!(
            updates,
            &[GestureUpdate {
                index: 0,
                begin_gesture: false,
                set_value: Some(0.0),
                end_gesture: false,
            }]
        );

        gestures.end_gesture(0);

        let updates = gestures.poll(&mut states).collect::<Vec<_>>();
        assert_eq!(
            updates,
            &[GestureUpdate {
                index: 0,
                begin_gesture: false,
                set_value: None,
                end_gesture: true,
            }]
        );

        gestures.begin_gesture(0);
        gestures.end_gesture(0);

        let updates = gestures.poll(&mut states).collect::<Vec<_>>();
        assert_eq!(
            updates,
            &[GestureUpdate {
                index: 0,
                begin_gesture: false,
                set_value: None,
                end_gesture: false,
            }]
        );

        gestures.begin_gesture(0);
        gestures.set_value(0, 1.0);

        let updates = gestures.poll(&mut states).collect::<Vec<_>>();
        assert_eq!(
            updates,
            &[GestureUpdate {
                index: 0,
                begin_gesture: true,
                set_value: Some(1.0),
                end_gesture: false,
            }]
        );

        gestures.set_value(0, 2.0);
        gestures.end_gesture(0);

        let updates = gestures.poll(&mut states).collect::<Vec<_>>();
        assert_eq!(
            updates,
            &[GestureUpdate {
                index: 0,
                begin_gesture: false,
                set_value: Some(2.0),
                end_gesture: true,
            }]
        );

        gestures.begin_gesture(0);
        gestures.set_value(0, 3.0);
        gestures.end_gesture(0);

        let updates = gestures.poll(&mut states).collect::<Vec<_>>();
        assert_eq!(
            updates,
            &[GestureUpdate {
                index: 0,
                begin_gesture: true,
                set_value: Some(3.0),
                end_gesture: true,
            }]
        );

        gestures.begin_gesture(0);
        gestures.set_value(0, 4.0);
        gestures.set_value(0, 5.0);
        gestures.set_value(0, 6.0);
        gestures.end_gesture(0);

        let updates = gestures.poll(&mut states).collect::<Vec<_>>();
        assert_eq!(
            updates,
            &[GestureUpdate {
                index: 0,
                begin_gesture: true,
                set_value: Some(6.0),
                end_gesture: true,
            }]
        );

        gestures.begin_gesture(0);
        gestures.set_value(0, 7.0);
        gestures.end_gesture(0);
        gestures.begin_gesture(0);
        gestures.set_value(0, 8.0);
        gestures.end_gesture(0);
        gestures.begin_gesture(0);
        gestures.set_value(0, 9.0);
        gestures.end_gesture(0);

        let updates = gestures.poll(&mut states).collect::<Vec<_>>();
        assert_eq!(
            updates,
            &[GestureUpdate {
                index: 0,
                begin_gesture: true,
                set_value: Some(9.0),
                end_gesture: true,
            }]
        );

        let updates = gestures.poll(&mut states).collect::<Vec<_>>();
        assert!(updates.is_empty());
    }
}
