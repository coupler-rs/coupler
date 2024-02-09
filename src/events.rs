use std::ops::{Index, Range};
use std::slice;

use crate::params::{ParamId, ParamValue};

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
}

impl<'a> Events<'a> {
    #[inline]
    pub fn new(events: &'a [Event]) -> Events<'a> {
        Events { events }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[inline]
    pub fn get<I>(&self, index: usize) -> Option<&'a Event> {
        self.events.get(index)
    }

    #[inline]
    pub fn slice(&self, range: Range<usize>) -> Option<Events<'a>> {
        if let Some(events) = self.events.get(range) {
            Some(Events { events })
        } else {
            None
        }
    }
}

impl<'a> Index<usize> for Events<'a> {
    type Output = Event;

    fn index(&self, index: usize) -> &Event {
        &self.events[index]
    }
}

impl<'a> IntoIterator for Events<'a> {
    type Item = &'a Event;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self.events.into_iter(),
        }
    }
}

pub struct Iter<'a> {
    iter: slice::Iter<'a, Event>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Event;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
