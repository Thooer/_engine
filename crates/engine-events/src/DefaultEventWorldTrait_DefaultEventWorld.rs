use std::collections::HashMap;
use crate::{DefaultEventWorld, DefaultEventWorldTrait};

impl DefaultEventWorldTrait for DefaultEventWorld {
    fn new() -> Self {
        Self {
            containers: HashMap::new(),
        }
    }
}
