use std::marker::PhantomData;

use super::Buffers;
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
        if self.buffers.len == 0 && self.events.len() == 0 {
            return None;
        }

        let mut event_count = 0;
        let mut split = self.buffers.len;
        for event in self.events {
            if event.time > self.time {
                split = split.min((event.time - self.time) as usize);
                self.time = event.time;
                break;
            }

            event_count += 1;
        }

        let buffers = Buffers {
            raw: self.buffers.raw,
            len: split,
            _marker: PhantomData,
        };
        self.buffers.raw.offset += split;
        self.buffers.len -= split;

        let events = self.events.slice(..event_count).unwrap();
        self.events = self.events.slice(event_count..).unwrap();

        Some((buffers, events))
    }
}
