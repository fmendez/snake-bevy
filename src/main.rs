use std::{collections::LinkedList, default};

use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
    prelude::*,
    reflect::impl_from_reflect_value,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::WindowResolution,
};

use rand::prelude::*;

const WALL_THICKNESS: f32 = 10.0;
const LEFT_WALL: f32 = -550.0;
const RIGHT_WALL: f32 = 550.0;
const BOTTOM_WALL: f32 = -350.0;
const TOP_WALL: f32 = 350.0;

const WALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const STEP_SIZE: f32 = 10.0;

#[derive(Component)]
struct Collider;

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone)]
struct SnakeSegment {
    x: f32,
    y: f32,
    entity: Option<Entity>,
}

#[derive(Component)]
struct SnakeHead;

#[derive(Component)]
struct SnakeTail;

#[derive(Component)]
struct SnakeBodySegment;

#[derive(Resource)]
struct Snake {
    direction: Direction,
    body: LinkedList<SnakeSegment>,
    head: SnakeSegment,
    tail: SnakeSegment,
    entity: Option<Entity>,
    move_cooldown: Timer,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

impl Default for Snake {
    fn default() -> Self {
        let mut body = LinkedList::new();
        let x = 20.0;
        let mut y = 20.0;

        let head = SnakeSegment {
            x,
            y: y + STEP_SIZE,
            entity: None,
        };

        for i in 2..=20 {
            y += STEP_SIZE * (i as f32);
            body.push_back(SnakeSegment { x, y, entity: None });
        }

        let tail = SnakeSegment {
            x,
            y: y + (STEP_SIZE * 11.0),
            entity: None,
        };

        Snake {
            direction: Direction::Up,
            head,
            body,
            tail,
            entity: None,
            move_cooldown: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}

#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.0),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.0),
            WallLocation::Bottom => Vec2::new(0.0, BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0.0, TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;

        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

fn main() {
    App::new()
        .init_resource::<Snake>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (check_for_collisions))
        .add_systems(Update, move_snake)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut snake: ResMut<Snake>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(20.0, 20.0))),
            material: materials.add(Color::GREEN),
            transform: Transform::from_xyz(snake.head.x, snake.head.y, 0.0),
            ..default()
        },
        SnakeHead,
        Collider,
    ));

    for segment in snake.body.iter_mut() {
        segment.entity = Some(
            commands
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Rectangle::new(20.0, 20.0))),
                        material: materials.add(Color::GREEN),
                        transform: Transform::from_xyz(segment.x, segment.y, 0.0),
                        ..default()
                    },
                    SnakeBodySegment,
                ))
                .id(),
        );
    }

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(20.0, 20.0))),
            material: materials.add(Color::GREEN),
            transform: Transform::from_xyz(snake.tail.x, snake.tail.y, 0.0),
            ..default()
        },
        SnakeTail,
        Collider,
    ));
}

fn move_snake(
    mut commands: Commands,
    mut snake: ResMut<Snake>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    // mut transforms: Query<&mut Transform>,
    mut snake_head_query: Query<&mut Transform, (With<Collider>, With<SnakeHead>)>,
    mut snake_body_segment_query: Query<
        &mut Transform,
        (With<SnakeBodySegment>, Without<SnakeHead>),
    >,
) {
    if snake.move_cooldown.tick(time.delta()).finished() {
        let mut snake_head_transform = snake_head_query.single_mut();
        let mut moved = false;

        snake.move_cooldown.reset();
        let mut current_position = snake_head_transform.translation;
        let mut prev_position: Vec3 = Vec3::new(0., 0., 0.);

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            moved = true;
            snake_head_transform.translation.y -= STEP_SIZE;
        }

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            moved = true;
            snake_head_transform.translation.y += STEP_SIZE;
        }

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            moved = true;
            snake_head_transform.translation.x -= STEP_SIZE;
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            moved = true;
            snake_head_transform.translation.x += STEP_SIZE;
        }

        if moved {
            for mut snake_body_segments_transform in snake_body_segment_query.iter_mut() {
                prev_position = snake_body_segments_transform.translation;
                snake_body_segments_transform.translation.x = current_position.x;
                snake_body_segments_transform.translation.y = current_position.y;
                current_position = prev_position;
            }
        }
    }
}

fn check_for_collisions(
    mut commands: Commands,
    mut snake: ResMut<Snake>,
    mut snake_head_query: Query<(Entity, &Transform), (With<SnakeHead>, With<Collider>)>,
    collider_query: Query<(Entity, &Transform), (With<Collider>, Without<SnakeHead>)>,
) {
    for (snake_segment_entity, snake_segment_transform) in &snake_head_query {
        for (collider_entity, collider_transform) in &collider_query {
            let snake_segment_bounded = Aabb2d::new(
                snake_segment_transform.translation.truncate(),
                snake_segment_transform.scale.truncate() / 2.0,
            );
            let wall_bounded = Aabb2d::new(
                collider_transform.translation.truncate(),
                collider_transform.scale.truncate() / 2.0,
            );
            let collision = collided_with_wall(snake_segment_bounded, wall_bounded);
            if let Some(collision) = collision {
                println!(
                    "[{:?}]Collision registered on {:?}",
                    std::time::SystemTime::now(),
                    collision
                );
            }
        }
    }
}

fn collided_with_wall(snake_segment: Aabb2d, wall: Aabb2d) -> Option<Collision> {
    if !snake_segment.intersects(&wall) {
        return None;
    }

    let closest = wall.closest_point(snake_segment.center());

    let offset = snake_segment.center() - closest;

    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0.0 {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0.0 {
        Collision::Top
    } else {
        Collision::Bottom
    };
    Some(side)
}
