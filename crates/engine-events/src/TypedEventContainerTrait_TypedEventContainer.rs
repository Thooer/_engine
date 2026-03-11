use crate::{Event, TypedEventContainerTrait, TypedEventContainer};

impl<T: Event> TypedEventContainerTrait<T> for TypedEventContainer<T> {
    fn new() -> Self {
        Self { events: Vec::new() }
    }
}
