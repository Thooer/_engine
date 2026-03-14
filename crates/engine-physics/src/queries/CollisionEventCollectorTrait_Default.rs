//! 默认碰撞事件收集器实现 - Default collision event collector implementation

use rapier3d::prelude::*;
use crate::PhysicsContext;
use crate::queries::CollisionEventCollectorTrait::CollisionEventCollectorTrait;
use crate::queries::CollisionEventCollectorTrait::{CollisionEvent, CollisionEventType};
use std::collections::HashSet;

struct DefaultCollisionEventCollector {
    previous_contacts: HashSet<(u32, u32)>,
    current_contacts: HashSet<(u32, u32)>,
}

impl CollisionEventCollectorTrait for DefaultCollisionEventCollector {
    fn new() -> Self {
        Self {
            previous_contacts: HashSet::new(),
            current_contacts: HashSet::new(),
        }
    }

    fn collect_events(&mut self, context: &PhysicsContext) -> Vec<CollisionEvent> {
        let mut events = Vec::new();
        
        std::mem::swap(&mut self.previous_contacts, &mut self.current_contacts);
        self.current_contacts.clear();
        
        for contact_pair in context.narrow_phase.contact_pairs() {
            let collider_a = contact_pair.collider1;
            let collider_b = contact_pair.collider2;
            
            let body_a = match context.collider_set.get(collider_a) {
                Some(c) => c.parent(),
                None => continue,
            };
            let body_b = match context.collider_set.get(collider_b) {
                Some(c) => c.parent(),
                None => continue,
            };
            
            let (handle_a, handle_b) = match (body_a, body_b) {
                (Some(a), Some(b)) => (a, b),
                _ => continue,
            };
            
            let idx_a = handle_a.into_raw_parts().0;
            let idx_b = handle_b.into_raw_parts().0;
            let pair = if idx_a < idx_b { (idx_a, idx_b) } else { (idx_b, idx_a) };
            
            self.current_contacts.insert(pair);
            
            let was_contained = self.previous_contacts.contains(&pair);
            let is_contained = self.current_contacts.contains(&pair);
            
            if !was_contained && is_contained {
                events.push(CollisionEvent {
                    event_type: CollisionEventType::Started,
                    body_handle_a: handle_a,
                    body_handle_b: handle_b,
                    collider_handle_a: collider_a,
                    collider_handle_b: collider_b,
                });
            } else if was_contained && is_contained {
                events.push(CollisionEvent {
                    event_type: CollisionEventType::Stay,
                    body_handle_a: handle_a,
                    body_handle_b: handle_b,
                    collider_handle_a: collider_a,
                    collider_handle_b: collider_b,
                });
            }
        }
        
        for pair in self.previous_contacts.difference(&self.current_contacts) {
            let (idx_a, idx_b) = *pair;
            let handle_a = RigidBodyHandle::from_raw_parts(idx_a, 0);
            let handle_b = RigidBodyHandle::from_raw_parts(idx_b, 0);
            events.push(CollisionEvent {
                event_type: CollisionEventType::Ended,
                body_handle_a: handle_a,
                body_handle_b: handle_b,
                collider_handle_a: ColliderHandle::invalid(),
                collider_handle_b: ColliderHandle::invalid(),
            });
        }
        
        events
    }

    fn collect_contacts(&self, context: &PhysicsContext) -> Vec<(RigidBodyHandle, RigidBodyHandle)> {
        let mut contacts = Vec::new();
        
        for contact_pair in context.narrow_phase.contact_pairs() {
            let collider_a = contact_pair.collider1;
            let collider_b = contact_pair.collider2;
            
            if let Some(body_a) = context.collider_set.get(collider_a).and_then(|c| c.parent()) {
                if let Some(body_b) = context.collider_set.get(collider_b).and_then(|c| c.parent()) {
                    contacts.push((body_a, body_b));
                }
            }
        }
        
        contacts
    }
}
