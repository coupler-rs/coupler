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
}

impl<'a> Events<'a> {
    pub fn new(events: &'a [Event]) -> Events<'a> {
        Events { events }
    }
}

impl<'a> IntoIterator for Events<'a> {
    type Item = Event;
    type IntoIter = Iter<'a>;

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
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }
}
