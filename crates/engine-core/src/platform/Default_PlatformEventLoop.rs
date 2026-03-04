//! Default trait 实现

use crate::platform::PlatformEventLoop;

impl Default for PlatformEventLoop {
    fn default() -> Self {
        Self {}
    }
}
