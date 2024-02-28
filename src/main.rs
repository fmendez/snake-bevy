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

#[derive(Component)]
struct Collider;

#[derive(Default)]
enum Direction {
    #[default]
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

#[derive(Resource, Default)]
struct Snake {
    direction: Direction,
    body: LinkedList<SnakeSegment>,
    tail: Option<SnakeSegment>,
    entity: Option<Entity>,
}

impl Snake {
    fn new(x: f32, y: f32) -> Snake {
        let mut body = LinkedList::new();
        body.push_back(SnakeSegment {
            x: x + 50.0,
            y,
            entity: None,
        });
        body.push_back(SnakeSegment {
            x: x + 20.0,
            y,
            entity: None,
        });
        body.push_back(SnakeSegment { x, y, entity: None });

        Snake {
            direction: Direction::Up,
            body,
            tail: None,
            entity: None,
        }
    }

    fn head_position(&self) -> (f32, f32) {
        let head_segment = self.body.front().unwrap();
        (head_segment.x, head_segment.y)
    }

    fn move_forward(&mut self, direction: Option<Direction>) {
        if let Some(d) = direction {
            self.direction = d;
        }

        let (head_position_last_x, head_position_last_y) = self.head_position();

        let new_snake_segment = match self.direction {
            Direction::Up => SnakeSegment {
                x: head_position_last_x,
                y: head_position_last_y - 1.0,
                entity: None,
            },
            Direction::Down => SnakeSegment {
                x: head_position_last_x,
                y: head_position_last_y + 1.0,
                entity: None,
            },
            Direction::Left => SnakeSegment {
                x: head_position_last_x - 1.0,
                y: head_position_last_y,
                entity: None,
            },
            Direction::Right => SnakeSegment {
                x: head_position_last_x + 1.0,
                y: head_position_last_y,
                entity: None,
            },
        };

        self.body.push_front(new_snake_segment);
        let removed_segment = self.body.pop_back().unwrap();
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
        // .add_systems(FixedUpdate, move_planet)
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

    *snake = Snake::new(20.0, 20.0);
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

// fn apple_setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
//     commands.spawn(Camera2dBundle::default());

//     commands.spawn((
//         MaterialMesh2dBundle {
//             mesh: Mesh2dHandle(meshes.add(Rectangle::new(20.0, 20.0))),
//             material: materials.add(Color::RED),
//             transform: Transform::from_xyz(0.0, 0.0, 0.0),
//             ..default()
//         },
//         Planet,
//     ));
// }
// fn move_planet(
//     mut query: Query<(&mut Transform, &mut Planet)>,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     mut windows: Query<&mut Window>,
//     time: Res<Time>,
// ) {
//     if keyboard_input.pressed(KeyCode::KeyR) {
//         let mut rng = thread_rng();
//         if let Some((mut transform, _planet)) = query.iter_mut().next() {
//             if rng.gen() {
//                 println!("after gen()");
//                 let window = windows.single_mut();
//                 let max_x = window.resolution.physical_width() as i32;
//                 let max_y = window.resolution.physical_height() as i32;
//                 let x = rng.gen_range(-max_x..max_x) as f32;
//                 let y = rng.gen_range(-max_y..max_y) as f32;
//                 println!("{}, {}", x, y);
//                 transform.translation.x = x * time.delta_seconds();
//                 transform.translation.y = y * time.delta_seconds();
//             }
//         }
//     }
// }
