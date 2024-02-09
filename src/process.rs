use crate::buffers::Buffers;
use crate::bus::Layout;
use crate::events::Events;
use crate::params::{ParamId, ParamValue};

#[derive(Clone)]
pub struct Config {
    pub layout: Layout,
    pub sample_rate: f64,
    pub max_buffer_size: usize,
}

pub struct Block<'a, 'b> {
    pub buffers: Buffers<'a>,
    pub events: Events<'b>,
}

pub trait Processor: Send + Sized + 'static {
    fn set_param(&mut self, id: ParamId, value: ParamValue);
    fn reset(&mut self);
    fn process(&mut self, block: Block);
}

impl<'a, 'b> Block<'a, 'b> {
    #[inline]
    pub fn split_at_events(self) -> SplitAtEvents<'a, 'b> {
        SplitAtEvents {
            buffers: self.buffers,
            events: self.events,
            offset: 0,
            event: 0,
        }
    }
}

pub struct SplitAtEvents<'a, 'b> {
    buffers: Buffers<'a>,
    events: Events<'b>,
    offset: i64,
    event: usize,
}

impl<'a, 'b> SplitAtEvents<'a, 'b> {
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
        let events = self.events.slice(self.event..next_event).unwrap();

        self.offset = next_offset;
        self.event = next_event;

        Some(Block { buffers, events })
    }
}
