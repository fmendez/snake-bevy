use std::collections::LinkedList;

use bevy::{
    math::{
        bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
        vec2,
    },
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use rand::prelude::*;

const WALL_THICKNESS: f32 = 10.0;
const LEFT_WALL: f32 = -350.0;
const RIGHT_WALL: f32 = 350.0;
const BOTTOM_WALL: f32 = -350.0;
const TOP_WALL: f32 = 350.0;

const WALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const STEP_SIZE: f32 = 1.0;
const STEP_VELOCITY: f32 = 800.0;
const SNAKE_HEAD_HITBOX: Vec2 = vec2(20.0, 20.0);

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
struct Apple;

#[derive(Component)]
struct SnakeBodySegment;

#[derive(Resource)]
struct Snake {
    direction: Direction,
    body: LinkedList<SnakeSegment>,
    head: SnakeSegment,
    entity: Option<Entity>,
    move_cooldown: Timer,
}

#[derive(Resource, Default)]
struct Scoreboard {
    score: u32,
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

        for i in 2..=4 {
            y += STEP_SIZE * (i as f32);
            body.push_back(SnakeSegment { x, y, entity: None });
        }

        Snake {
            direction: Direction::Up,
            head,
            body,
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
        .init_resource::<Scoreboard>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (check_for_collisions, score_update))
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

    snake_spawn(&mut commands, &mut meshes, &mut materials);
    apple_spawn(&mut commands, &mut meshes, &mut materials);

    // scoreboard
    commands.spawn(
        TextBundle::from_section(
            "Apples Eaten: 0",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}

fn move_snake(
    mut snake: ResMut<Snake>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
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
        let movement_amount = STEP_SIZE * STEP_VELOCITY * time.delta_seconds();

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            moved = true;
            snake_head_transform.translation.y -= movement_amount;
        }

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            moved = true;
            snake_head_transform.translation.y += movement_amount;
        }

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            moved = true;
            snake_head_transform.translation.x -= movement_amount;
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            moved = true;
            snake_head_transform.translation.x += movement_amount;
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    snake_head_query: Query<(Entity, &Transform), (With<SnakeHead>, With<Collider>)>,
    collider_query: Query<
        (Entity, &Transform, Option<&Apple>),
        (With<Collider>, Without<SnakeHead>),
    >,
) {
    for (snake_segment_entity, snake_head_transform) in &snake_head_query {
        for (collider_entity, collider_transform, maybe_apple) in &collider_query {
            let snake_head_bounded = Aabb2d::new(
                snake_head_transform.translation.truncate(),
                SNAKE_HEAD_HITBOX / 2.0,
            );
            let hitbox = if maybe_apple.is_some() {
                SNAKE_HEAD_HITBOX / 2.0
            } else {
                collider_transform.scale.truncate() / 2.0
            };

            let wall_or_apple_bounded =
                Aabb2d::new(collider_transform.translation.truncate(), hitbox);
            let collision = collided_with_wall_apple(snake_head_bounded, wall_or_apple_bounded);
            if let Some(collision) = collision {
                if maybe_apple.is_some() {
                    scoreboard.score += 1;
                    commands.get_entity(collider_entity).unwrap().despawn();
                    apple_spawn(&mut commands, &mut meshes, &mut materials);
                    snake_segment_spawn(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        snake_head_transform.translation.x,
                        snake_head_transform.translation.y,
                    );
                } else {
                    println!(
                        "[{:?}]Collision with Wall on {:?}",
                        std::time::SystemTime::now(),
                        collision
                    );
                }
            }
        }
    }
}

fn collided_with_wall_apple(snake_segment: Aabb2d, wall_or_apple: Aabb2d) -> Option<Collision> {
    if !snake_segment.intersects(&wall_or_apple) {
        return None;
    }

    let closest = wall_or_apple.closest_point(snake_segment.center());

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

fn apple_rng_position() -> Vec3 {
    let mut rng = thread_rng();
    let x = rng.gen_range((LEFT_WALL + WALL_THICKNESS)..(RIGHT_WALL - WALL_THICKNESS)) as f32;
    let y = rng.gen_range((BOTTOM_WALL + WALL_THICKNESS)..(TOP_WALL - WALL_THICKNESS)) as f32;

    let z = -2.0;
    Vec3 { x, y, z }
}

fn apple_spawn(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let apple_pos = apple_rng_position();
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(20.0, 20.0))),
            material: materials.add(Color::RED),
            transform: Transform::from_xyz(apple_pos.x, apple_pos.y, apple_pos.z),
            ..default()
        },
        Apple,
        Collider,
    ));
}

fn snake_segment_spawn(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    x: f32,
    y: f32,
) -> Entity {
    commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(20.0, 20.0))),
                material: materials.add(Color::GREEN),
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            SnakeBodySegment,
        ))
        .id()
}

fn snake_spawn(
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let mut snake = Snake::default();

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
        segment.entity = Some(snake_segment_spawn(
            &mut commands,
            &mut meshes,
            &mut materials,
            segment.x,
            segment.y,
        ));
    }
}

fn score_update(mut scoreboard: ResMut<Scoreboard>, mut query: Query<&mut Text>) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("Apples Eaten: {}", scoreboard.score);
    }
}
