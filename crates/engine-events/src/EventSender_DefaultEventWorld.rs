use std::any::TypeId;
use crate::{Event, DefaultEventWorld, TypedEventContainer, TypedEventContainerTrait, EventSender};

impl EventSender for DefaultEventWorld {
    fn send_event<T: Event + Clone>(&mut self, event: T) {
        let type_id = TypeId::of::<T>();
        
        if !self.containers.contains_key(&type_id) {
            self.containers.insert(type_id, Box::new(TypedEventContainer::<T>::new()));
        }
        
        let container = self.containers
            .get_mut(&type_id)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<TypedEventContainer<T>>()
            .unwrap();
        
        container.events.push(event);
    }
}
