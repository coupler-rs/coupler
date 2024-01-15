use std::slice;

use crate::{ParamId, ParamValue};

#[derive(Copy, Clone)]
pub struct Event {
    pub time: i64,
    pub data: Data,
}

#[derive(Copy, Clone)]
#[non_exhaustive]
pub enum Data {
    ParamChange { id: ParamId, value: ParamValue },
}

#[derive(Copy, Clone)]
pub struct Events<'a> {
    events: &'a [Event],
    offset: i64,
}

impl<'a> Events<'a> {
    pub fn new(events: &'a [Event], offset: i64) -> Events<'a> {
        Events { events, offset }
    }
}

impl<'a> IntoIterator for Events<'a> {
    type Item = Event;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self.events.into_iter(),
            offset: self.offset,
        }
    }
}

pub struct Iter<'a> {
    iter: slice::Iter<'a, Event>,
    offset: i64,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&Event { time, data }) = self.iter.next() {
            Some(Event {
                time: time + self.offset,
                data,
            })
        } else {
            None
        }
    }
}
