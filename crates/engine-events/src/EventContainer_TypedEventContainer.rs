use std::any::Any;
use crate::{Event, TypedEventContainer, EventContainer};

impl<T: Event> EventContainer for TypedEventContainer<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
