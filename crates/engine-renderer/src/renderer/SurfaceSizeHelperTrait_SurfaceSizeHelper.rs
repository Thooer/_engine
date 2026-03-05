use super::{SurfaceSize, SurfaceSizeHelper, SurfaceSizeHelperTrait};

impl SurfaceSizeHelperTrait for SurfaceSizeHelper {
    fn surface_size_is_zero(size: SurfaceSize) -> bool {
        size.width == 0 || size.height == 0
    }
}
