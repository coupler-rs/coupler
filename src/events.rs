use std::ops::Range;
use std::slice;

use crate::{ParamId, ParamValue};

#[derive(Copy, Clone)]
pub struct Event {
    pub time: i64,
    pub data: Data,
}

impl Event {
    #[inline]
    fn offset(self, offset: i64) -> Event {
        Event {
            time: self.time + offset,
            data: self.data,
        }
    }
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
    #[inline]
    pub fn new(events: &'a [Event], offset: i64) -> Events<'a> {
        Events { events, offset }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[inline]
    pub fn get<I>(&self, index: usize) -> Option<Event> {
        if let Some(&event) = self.events.get(index) {
            Some(event.offset(self.offset))
        } else {
            None
        }
    }

    #[inline]
    pub fn slice(&self, range: Range<usize>) -> Option<Events<'a>> {
        if let Some(events) = self.events.get(range) {
            Some(Events {
                events,
                offset: self.offset,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn with_offset(&self, offset: i64) -> Events<'a> {
        Events {
            events: self.events,
            offset: self.offset + offset,
        }
    }
}

impl<'a> IntoIterator for Events<'a> {
    type Item = Event;
    type IntoIter = Iter<'a>;

    #[inline]
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

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&event) = self.iter.next() {
            Some(event.offset(self.offset))
        } else {
            None
        }
    }
}
