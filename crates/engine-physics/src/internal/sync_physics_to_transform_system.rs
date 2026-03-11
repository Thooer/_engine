{
    use glam::Vec3;

    let rigid_body_set = &physics_context.rigid_body_set;

    for (handle, mut transform) in transform_query.iter_mut() {
        if let Some(rb) = rigid_body_set.get(handle.rigid_body_handle) {
            let pos = rb.translation();
            transform.translation = Vec3::new(pos.x, pos.y, pos.z);
        }
    }
}
