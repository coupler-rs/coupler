use crate::buffers::Buffers;
use crate::events::Events;

pub struct Block<'a, 'b, 'c> {
    pub buffers: Buffers<'a, 'b>,
    pub events: Events<'c>,
}
