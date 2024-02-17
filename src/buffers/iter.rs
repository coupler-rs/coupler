use std::marker::PhantomData;

use super::{Offset, SampleBuffer};
use crate::events::Events;

pub struct SplitAtEvents<'e, B: SampleBuffer> {
    raw: B::Raw,
    len: usize,
    events: Events<'e>,
    time: i64,
    _marker: PhantomData<B>,
}

impl<'e, B: SampleBuffer> SplitAtEvents<'e, B> {
    #[inline]
    pub(crate) fn new(buffer: B, events: Events<'e>) -> SplitAtEvents<'e, B> {
        let (raw, len) = buffer.into_raw_parts();

        SplitAtEvents {
            raw,
            len,
            events,
            time: 0,
            _marker: PhantomData,
        }
    }
}

impl<'e, B: SampleBuffer> Iterator for SplitAtEvents<'e, B> {
    type Item = (B, Events<'e>);

    #[inline]
    fn next(&mut self) -> Option<(B, Events<'e>)> {
        if self.len == 0 {
            if self.events.len() == 0 {
                return None;
            }

            // If we've reached the end of the buffer, yield all remaining events in one go:
            let buffers = unsafe { B::from_raw_parts(self.raw, 0) };

            let events = self.events;
            self.events = Events::new(&[]);

            return Some((buffers, events));
        }

        // Find the first event with a timestamp greater than the current one:
        let mut event_count = 0;
        let mut split = self.len;
        for event in self.events {
            if event.time > self.time {
                let offset = (event.time - self.time) as usize;
                if offset < self.len {
                    split = offset;
                }

                self.time = event.time;

                break;
            }

            event_count += 1;
        }

        let buffer = unsafe { B::from_raw_parts(self.raw, split) };
        self.raw = unsafe { self.raw.offset(split as isize) };
        self.len -= split;

        let events = self.events.slice(..event_count).unwrap();
        self.events = self.events.slice(event_count..).unwrap();

        Some((buffer, events))
    }
}
