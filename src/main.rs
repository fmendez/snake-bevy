use std::{collections::LinkedList, default};

use bevy::{
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

const STEP_SIZE: f32 = 20.0;

#[derive(Component)]
struct Collider;

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct SnakeSegment {
    x: f32,
    y: f32,
    entity: Option<Entity>,
}

#[derive(Resource)]
struct Snake {
    direction: Direction,
    body: LinkedList<SnakeSegment>,
    tail: Option<SnakeSegment>,
    entity: Option<Entity>,
    move_cooldown: Timer,
}

impl Default for Snake {
    fn default() -> Self {
        let mut body = LinkedList::new();
        let mut x = 20.0;
        let mut y = 20.0;

        for _ in 0..3 {
            y += STEP_SIZE;
            body.push_back(SnakeSegment { x, y, entity: None });
        }

        Snake {
            direction: Direction::Up,
            body,
            tail: None,
            entity: None,
            move_cooldown: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}

impl Snake {
    fn head_position(&self) -> (f32, f32) {
        let head_segment = self.body.front().unwrap();
        (head_segment.x, head_segment.y)
    }

    fn move_forward(
        &mut self,
        direction: Option<Direction>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        if let Some(d) = direction {
            self.direction = d;
        }

        let (head_position_last_x, head_position_last_y) = self.head_position();

        let mut new_snake_segment = match self.direction {
            Direction::Up => SnakeSegment {
                x: head_position_last_x,
                y: head_position_last_y + STEP_SIZE,
                entity: None,
            },
            Direction::Down => SnakeSegment {
                x: head_position_last_x,
                y: head_position_last_y - STEP_SIZE,
                entity: None,
            },
            Direction::Left => SnakeSegment {
                x: head_position_last_x - STEP_SIZE,
                y: head_position_last_y,
                entity: None,
            },
            Direction::Right => SnakeSegment {
                x: head_position_last_x + STEP_SIZE,
                y: head_position_last_y,
                entity: None,
            },
        };

        new_snake_segment.entity = Some(
            commands
                .spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::new(20.0, 20.0))),
                    material: materials.add(Color::GREEN),
                    transform: Transform::from_xyz(new_snake_segment.x, new_snake_segment.y, 0.0),
                    ..default()
                })
                .id(),
        );
        self.body.push_front(new_snake_segment);
        let removed_segment = self.body.pop_back().unwrap();
        commands
            .get_entity(removed_segment.entity.unwrap())
            .unwrap()
            .despawn();
        self.tail = Some(removed_segment);
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

    for segment in snake.body.iter_mut() {
        segment.entity = Some(
            commands
                .spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::new(20.0, 20.0))),
                    material: materials.add(Color::GREEN),
                    transform: Transform::from_xyz(segment.x, segment.y, 0.0),
                    ..default()
                })
                .id(),
        );
    }
}

fn move_snake(
    mut commands: Commands,
    mut snake: ResMut<Snake>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut transforms: Query<&mut Transform>,
) {
    if snake.move_cooldown.tick(time.delta()).finished() {
        let mut moved = false;
        let mut direction: Direction = Direction::Down;

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            direction = Direction::Down;
            moved = true;
        }

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction = Direction::Up;
            moved = true;
        }

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction = Direction::Left;
            moved = true;
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction = Direction::Right;
            moved = true;
        }

        if moved {
            snake.move_cooldown.reset();
            snake.move_forward(Some(direction), commands, meshes, materials)
        }
    }
}
