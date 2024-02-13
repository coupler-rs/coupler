use std::marker::PhantomData;

use super::Buffers;
use crate::events::Events;

pub struct SplitAtEvents<'a, 'b, 'e> {
    buffers: Buffers<'a, 'b>,
    events: Events<'e>,
    offset: i64,
}

impl<'a, 'b, 'e> SplitAtEvents<'a, 'b, 'e> {
    #[inline]
    pub(crate) fn new(buffers: Buffers<'a, 'b>, events: Events<'e>) -> SplitAtEvents<'a, 'b, 'e> {
        SplitAtEvents {
            buffers: buffers,
            events: events,
            offset: 0,
        }
    }
}

impl<'a, 'b, 'e> Iterator for SplitAtEvents<'a, 'b, 'e> {
    type Item = (Buffers<'a, 'b>, Events<'e>);

    #[inline]
    fn next(&mut self) -> Option<(Buffers<'a, 'b>, Events<'e>)> {
        if self.buffers.len() == 0 && self.events.len() == 0 {
            return None;
        }

        let mut event_count = 0;
        let mut split = self.buffers.len();
        for event in self.events {
            if event.time > self.offset {
                split = split.min((event.time - self.offset) as usize);
                break;
            }

            event_count += 1;
        }

        let buffers = Buffers {
            buffers: self.buffers.buffers,
            ptrs: self.buffers.ptrs,
            offset: self.buffers.offset,
            len: split,
            _marker: PhantomData,
        };
        self.buffers.offset += split;
        self.buffers.len -= split;

        let events = self.events.slice(..event_count).unwrap();
        self.events = self.events.slice(event_count..).unwrap();

        Some((buffers, events))
    }
}
