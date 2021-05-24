extern crate realm;


#[test]
fn test_empty_entity_store_is_empty() {
    let world = realm::World::new();

    assert!(world.is_empty());
}

#[test]
fn test_empty_entity_storage_has_zero_length() {
    let world = realm::World::new();

    assert_eq!(world.len(), 0);
}

#[test]
fn test_entity_storage_one_element() {
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }

    struct Velocity {
        x: f32,
        y: f32,
        z: f32,
    }

    let mut world = realm::World::new();
    let entity = world.push((
        Position {
            x: 0_f32,
            y: 0_f32,
            z: 0_f32,
        },
        Velocity {
            x: 1_f32,
            y: 1_f32,
            z: 1_f32,
        }
    ));

    assert!(!world.is_empty());
    assert!(world.contains(entity));
}

