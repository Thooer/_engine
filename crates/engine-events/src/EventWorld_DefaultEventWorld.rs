use std::any::TypeId;
use crate::{Event, EventWorld, DefaultEventWorld, TypedEventContainer, EventSender};

impl EventWorld for DefaultEventWorld {
    fn send<T: Event + Clone>(&mut self, event: T) {
        self.send_event(event);
    }

    fn read<T: Event + Clone>(&self) -> Vec<T> {
        let type_id = TypeId::of::<T>();
        
        if let Some(container) = self.containers.get(&type_id) {
            let typed = container.as_any().downcast_ref::<TypedEventContainer<T>>();
            if let Some(events) = typed {
                return events.events.clone();
            }
        }
        
        Vec::new()
    }

    fn clear<T: Event>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.containers.remove(&type_id);
    }
}
