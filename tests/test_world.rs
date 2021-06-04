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

struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Position {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: x,
            y: y,
            z: z,
        }
    }
}

struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

impl Velocity {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: x,
            y: y,
            z: z,
        }
    }
}

struct Acceleration {
    x: f32,
    y: f32,
    z: f32,
}

impl Acceleration {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: x,
            y: y,
            z: z,
        }
    }
}


#[test]
fn test_entity_storage_with_one_element_is_not_empty() {
    let mut world = realm::World::new();
    let _ = world.push((
        Position::new(0_f32, 0_f32, 0_f32),
        Velocity::new(1_f32, 1_f32, 1_f32)
    ));

    assert!(!world.is_empty());
}

#[test]
fn test_entity_storage_with_one_element_contains() {
    let mut world = realm::World::new();
    let entity = world.push((
        Position::new(0_f32, 0_f32, 0_f32),
        Velocity::new(1_f32, 1_f32, 1_f32)
    ));

    assert!(world.contains(entity));
}

#[test]
fn test_entity_storage_has_components() {
    let mut world = realm::World::new();
    let entity = world.push((
        Position::new(0_f32, 0_f32, 0_f32),
        Velocity::new(1_f32, 1_f32, 1_f32)
    ));

    assert!(world.has_component::<Position>(entity));
    assert!(world.has_component::<Velocity>(entity));
    assert!(!world.has_component::<Acceleration>(entity));
}

#[test]
fn test_entity_storage_contains_components() {
    let mut world = realm::World::new();
    let _ = world.push((
        Position::new(0_f32, 0_f32, 0_f32),
        Velocity::new(1_f32, 1_f32, 1_f32)
    ));

    assert!(world.contains_component::<Position>());
    assert!(world.contains_component::<Velocity>());
    assert!(!world.contains_component::<Acceleration>());
}

#[test]
fn test_entity_storage_push_multiple_elements() {
    let mut world = realm::World::new();
    let _ = world.push((
        Position::new(0_f32, 0_f32, 0_f32),
        Velocity::new(0_f32, 0_f32, 0_f32)
    ));
    let _ = world.push((
        Position::new(1_f32, 1_f32, 1_f32),
        Velocity::new(1_f32, 1_f32, 1_f32)
    ));
    let _ = world.push((
        Position::new(2_f32, 2_f32, 2_f32),
        Velocity::new(2_f32, 2_f32, 2_f32)
    ));

    assert_eq!(world.len(), 3);
}

#[test]
fn test_entity_storage_push_multiple_elements_clear() {
    let mut world = realm::World::new();
    let _ = world.push((
        Position::new(0_f32, 0_f32, 0_f32),
        Velocity::new(0_f32, 0_f32, 0_f32)
    ));
    let _ = world.push((
        Position::new(1_f32, 1_f32, 1_f32),
        Velocity::new(1_f32, 1_f32, 1_f32)
    ));
    let _ = world.push((
        Position::new(2_f32, 2_f32, 2_f32),
        Velocity::new(2_f32, 2_f32, 2_f32)
    ));

    world.clear();

    assert!(world.is_empty());
}

#[test]
fn test_entity_storage_push_multiple_elements_remove() {
    let mut world = realm::World::new();
    let entity0 = world.push((
        Position::new(0_f32, 0_f32, 0_f32),
        Velocity::new(0_f32, 0_f32, 0_f32)
    ));
    let entity1 = world.push((
        Position::new(1_f32, 1_f32, 1_f32),
        Velocity::new(1_f32, 1_f32, 1_f32)
    ));
    let entity2 = world.push((
        Position::new(2_f32, 2_f32, 2_f32),
        Velocity::new(2_f32, 2_f32, 2_f32)
    ));

    assert!(world.contains(entity0));
    assert!(world.contains(entity1));
    assert!(world.contains(entity2));

    world.remove(entity1);
    assert!(!world.contains(entity1));

    world.remove(entity2);
    assert!(!world.contains(entity2));

    world.remove(entity0);
    assert!(!world.contains(entity0));

    assert!(world.is_empty());
}

