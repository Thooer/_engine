{
    let rigid_body_set = &mut physics_context.rigid_body_set;

    for (transform, handle) in transform_query.iter_mut() {
        if let Some(body) = rigid_body_set.get_mut(handle.rigid_body_handle) {
            body.set_translation(transform.translation, true);
        }
    }
}
