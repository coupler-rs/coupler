use crate::buffers::Buffers;
use crate::events::Events;

pub struct Block<'a, 'b, 'c> {
    pub buffers: Buffers<'a, 'b>,
    pub events: Events<'c>,
}

impl<'a, 'b, 'c> Block<'a, 'b, 'c> {
    #[inline]
    pub fn split_at_events(self) -> SplitAtEvents<'a, 'b, 'c> {
        SplitAtEvents {
            buffers: self.buffers,
            events: self.events,
            offset: 0,
            event: 0,
        }
    }
}

pub struct SplitAtEvents<'a, 'b, 'c> {
    buffers: Buffers<'a, 'b>,
    events: Events<'c>,
    offset: i64,
    event: usize,
}

impl<'a, 'b, 'c> SplitAtEvents<'a, 'b, 'c> {
    #[inline]
    pub fn next(&mut self) -> Option<Block> {
        let end_offset = self.buffers.len() as i64;
        if self.offset == end_offset && self.event == self.events.len() {
            return None;
        }

        let mut next_offset = end_offset;
        let mut next_event = self.event;
        for event in self.events.slice(self.event..self.events.len()).unwrap() {
            if event.time >= end_offset {
                next_event = self.events.len();
                break;
            }

            if event.time > self.offset {
                next_offset = event.time;
                break;
            }

            next_event += 1;
        }

        let buffers = self.buffers.slice(self.offset as usize..next_offset as usize).unwrap();
        let events = self.events.slice(self.event..next_event).unwrap().with_offset(-self.offset);

        self.offset = next_offset;
        self.event = next_event;

        Some(Block { buffers, events })
    }
}
