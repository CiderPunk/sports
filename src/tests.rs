//AI Disclosure: GEMINI wrote these

#[cfg(test)]
mod tests {
    use crate::collisions::SphereCast;
    use bevy::prelude::*;

    // Mock setup helper to avoid boilerplate across tests
    fn setup_cast(origin: Vec3, direction: Vec3, radius: f32, max_distance: f32) -> SphereCast {
        SphereCast {
            origin,
            direction: Dir3::new(direction).expect("Direction must be a non-zero unit vector"),
            radius,
            distance: max_distance,
        }
    }



    #[test]
    fn odd_real_case() {
        // A sphere cast starting at (0, 0, 0) moving straight along the X axis
        let cast = setup_cast(Vec3::new(-16.543943, 0.7553604, -2.5560455), Vec3::new(-0.8203597, 0.15396476, -0.5507311), 0.125, 0.019819815);
        
        
        let target_pos = Vec3::new(-13.172179, 0., -0.7266768); // y=-1 to y=1 spans the ray at y=0
        let target_radius = 0.5;
        let target_height = 1.8;
        let target_entity = Entity::from_raw_u32(1).unwrap();

        let result = cast.interset_sphere_vertical_cylinder(
            target_pos,
            target_radius,
            target_height,
            target_entity,
        );

        assert!(result.is_none(), "The ray should not hit the cylinder footprint.");
				
    }



    #[test]
    fn test_perfect_horizontal_hit() {
        // A sphere cast starting at (0, 0, 0) moving straight along the X axis
        let cast = setup_cast(Vec3::ZERO, Vec3::X, 0.5, 20.0);
        
        // Target cylinder placed 10 units away on the X axis, height matching the ray
        let target_pos = Vec3::new(10.0, -1.0, 0.0); // y=-1 to y=1 spans the ray at y=0
        let target_radius = 0.5;
        let target_height = 2.0;
        let target_entity = Entity::from_raw_u32(1).unwrap();

        let result = cast.interset_sphere_vertical_cylinder(
            target_pos,
            target_radius,
            target_height,
            target_entity,
        );

        assert!(result.is_some(), "The ray should hit the cylinder footprint.");
        let hit = result.unwrap();
        
        // Combined radius = 0.5 + 0.5 = 1.0. 
        // Cylinder is at X=10, so hit should register at exactly X = 10 - 1 = 9.0.
        assert_eq!(hit.entity, target_entity);
        assert!((hit.distance - 9.0).abs() < 1e-4, "Distance should be exactly 9.0");
        assert!((hit.position.x - 9.0).abs() < 1e-4, "Hit position X should be 9.0");
        
        // Normal should point directly back along the negative X axis
        assert!((*hit.normal - Vec3::NEG_X).length() < 1e-4, "Normal should face NEG_X");
    }

    #[test]
    fn test_angled_3d_hit() {
        // Ray shooting diagonally upward at a 45-degree angle on the XY plane
        let direction = Vec3::new(1.0, 1.0, 0.0).normalize();
        let cast = setup_cast(Vec3::ZERO, direction, 0.0, 50.0); // Zero radius for raw ray tracking

        // Target cylinder is at X=10, Z=0. Ground floor at Y=5, ceiling at Y=15
        let target_pos = Vec3::new(10.0, 5.0, 0.0);
        let target_radius = 1.0;
        let target_height = 10.0;

        let result = cast.interset_sphere_vertical_cylinder(
            target_pos,
            target_radius,
            target_height,
						Entity::from_raw_u32(2).unwrap(),
        );

        assert!(result.is_some(), "The upward angled ray should hit the cylinder wall.");
        let hit = result.unwrap();

        // Ray hits the 2D boundary (X = 10 - 1 = 9.0). 
        // At X=9 on a 45-degree trajectory, Y must also be 9.0. 
        // Y=9 is perfectly within the cylinder height range (5.0 to 15.0).
        assert!((hit.position.x - 9.0).abs() < 1e-4);
        assert!((hit.position.y - 9.0).abs() < 1e-4);
    }

    #[test]
    fn test_height_miss_over_the_top() {
        // Ray shoots diagonally upward like the test above
        let direction = Vec3::new(1.0, 1.0, 0.0).normalize();
        let cast = setup_cast(Vec3::ZERO, direction, 0.0, 50.0);

        // Target cylinder is at X=10, but its height profile is low (Y=0 to Y=4)
        let target_pos = Vec3::new(10.0, 0.0, 0.0);
        let target_radius = 1.0;
        let target_height = 4.0;

        let result = cast.interset_sphere_vertical_cylinder(
            target_pos,
            target_radius,
            target_height,
            		Entity::from_raw_u32(3).unwrap(),
        );

        // Horizontally it passes through the footprint, but it sails over the top on the Y axis
        assert!(result.is_none(), "The ray should pass cleanly over the cylinder cap.");
    }

    #[test]
    fn test_out_of_range_miss() {
        let cast = setup_cast(Vec3::ZERO, Vec3::X, 0.5, 5.0); // Max distance constraint is set short (5.0 units)
        
        let target_pos = Vec3::new(10.0, -1.0, 0.0); // Cylinder is 10 units away
        let target_radius = 0.5;
        let target_height = 2.0;

        let result = cast.interset_sphere_vertical_cylinder(
            target_pos,
            target_radius,
            target_height,
            		Entity::from_raw_u32(4).unwrap(),
        );

        assert!(result.is_none(), "The ray should fall short of reaching the target.");
    }

    #[test]
    fn test_initial_overlap_edge_case() {
        // The ray starts directly INSIDE the horizontal footprint of the cylinder
        let cast = setup_cast(Vec3::ZERO, Vec3::X, 0.5, 10.0);
        
        let target_pos = Vec3::new(0.2, -1.0, 0.0); // Distance to target center is only 0.2 units
        let target_radius = 1.0;
        let target_height = 2.0;

        let result = cast.interset_sphere_vertical_cylinder(
            target_pos,
            target_radius,
            target_height,
            		Entity::from_raw_u32(5).unwrap(),
        );

        // Your code handles initial overlaps smoothly because `t_horiz_entry` becomes negative, 
        // causing `distance` to evaluate properly against boundaries.
        assert!(result.is_some(), "Initial boundary overlaps should register hits safely.");
    }
/*
    #[test]
    fn test_dot_product_pre_filter() {
        let cast = setup_cast(Vec3::ZERO, Vec3::X, 0.5, 20.0); // Moving East (+X)
        
        let target_behind = Vec3::new(-5.0, 0.0, 0.0); // Target is standing West (-X)
        let target_in_front = Vec3::new(5.0, 0.0, 0.0);

        assert!(cast.is_target_in_front_2d(target_in_front), "Target in front should pass.");
        assert!(!cast.is_target_in_front_2d(target_behind), "Target behind should fail.");
    }
		 */
}
