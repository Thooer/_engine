{
    let rigid_body_set = &mut physics_context.rigid_body_set;

    for (handle, force) in force_query.iter() {
        if let Some(body) = rigid_body_set.get_mut(handle.rigid_body_handle) {
            body.add_force(force.force, true);
            body.add_torque(force.torque, true);
        }
    }
}
