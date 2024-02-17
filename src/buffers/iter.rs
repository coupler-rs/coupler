use super::{Buffers, SampleBuffer};
use crate::events::Events;

pub struct SplitAtEvents<'a, 'b, 'e> {
    buffers: Buffers<'a, 'b>,
    events: Events<'e>,
    time: i64,
}

impl<'a, 'b, 'e> SplitAtEvents<'a, 'b, 'e> {
    #[inline]
    pub(crate) fn new(buffers: Buffers<'a, 'b>, events: Events<'e>) -> SplitAtEvents<'a, 'b, 'e> {
        SplitAtEvents {
            buffers: buffers,
            events: events,
            time: 0,
        }
    }
}

impl<'a, 'b, 'e> Iterator for SplitAtEvents<'a, 'b, 'e> {
    type Item = (Buffers<'a, 'b>, Events<'e>);

    #[inline]
    fn next(&mut self) -> Option<(Buffers<'a, 'b>, Events<'e>)> {
        if self.buffers.len == 0 {
            if self.events.len() == 0 {
                return None;
            }

            // If we've reached the end of the buffer, yield all remaining events in one go:
            let buffers = unsafe { Buffers::from_raw_parts(self.buffers.raw, 0) };

            let events = self.events;
            self.events = Events::new(&[]);

            return Some((buffers, events));
        }

        // Find the first event with a timestamp greater than the current one:
        let mut event_count = 0;
        let mut split = self.buffers.len;
        for event in self.events {
            if event.time > self.time {
                let offset = (event.time - self.time) as usize;
                if offset < self.buffers.len {
                    split = offset;
                }

                self.time = event.time;

                break;
            }

            event_count += 1;
        }

        let buffers = unsafe { Buffers::from_raw_parts(self.buffers.raw, split) };
        self.buffers.raw.offset += split;
        self.buffers.len -= split;

        let events = self.events.slice(..event_count).unwrap();
        self.events = self.events.slice(event_count..).unwrap();

        Some((buffers, events))
    }
}
