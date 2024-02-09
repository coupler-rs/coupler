use std::marker::PhantomData;

use super::Buffers;
use crate::events::Events;

pub struct SplitAtEvents<'a, 'b> {
    buffers: Buffers<'a>,
    events: Events<'b>,
    offset: i64,
}

impl<'a, 'b> SplitAtEvents<'a, 'b> {
    #[inline]
    pub(crate) fn new(buffers: Buffers<'a>, events: Events<'b>) -> SplitAtEvents<'a, 'b> {
        SplitAtEvents {
            buffers: buffers,
            events: events,
            offset: 0,
        }
    }
}

impl<'a, 'b> Iterator for SplitAtEvents<'a, 'b> {
    type Item = (Buffers<'a>, Events<'b>);

    #[inline]
    fn next(&mut self) -> Option<(Buffers<'a>, Events<'b>)> {
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
            buses: self.buffers.buses,
            ptrs: self.buffers.ptrs,
            offset: self.buffers.offset,
            len: split,
            _marker: PhantomData,
        };
        self.buffers.offset += split as isize;
        self.buffers.len -= split;

        let events = self.events.slice(..event_count).unwrap();
        self.events = self.events.slice(event_count..).unwrap();

        Some((buffers, events))
    }
}
