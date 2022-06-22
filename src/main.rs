use std::collections::VecDeque;

use bevy::core::FixedTimestep;
use bevy::prelude::*;
use rand::prelude::random;

const ARENA_WIDTH: u32 = 32;
const ARENA_HEIGHT: u32 = 18;

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 800.0;

const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(Component)]
struct InputBuffer {
    inputs: VecDeque<Direction>,
}

struct GameOverEvent;

struct GrowthEvent;

struct FoodEvent;

#[derive(Default)]
struct LastTailPosition(Option<Position>);

#[derive(Component)]
struct SnakeSegment;

#[derive(Default, Deref, DerefMut)]
struct SnakeSegments(Vec<Entity>);

#[derive(Component)]
struct Food;

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(SystemLabel, Clone, Hash, Debug, Eq, PartialEq)]
pub enum SnakeMovement {
    Input,
    Movement,
    Eating,
    Growth,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn setup_snake_game(
    mut commands: Commands,
    mut food_writer: EventWriter<FoodEvent>,
    mut segments: ResMut<SnakeSegments>,
) {
    *segments = SnakeSegments(vec![
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: SNAKE_HEAD_COLOR,
                    ..default()
                },
                ..default()
            })
            .insert(SnakeHead {
                direction: Direction::Up,
            })
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.8))
            .id(),
        spawn_segment(commands, Position { x: 3, y: 2 }),
    ]);
    food_writer.send(FoodEvent);
}

fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
}

fn spawn_food(mut commands: Commands, position: Position) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: FOOD_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Food)
        .insert(position)
        .insert(Size::square(0.8));
}

fn init_inputs(mut commands: Commands) {
    commands.spawn().insert(InputBuffer {
        inputs: VecDeque::with_capacity(10),
    });
}

fn snake_movement(
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_write: EventWriter<GameOverEvent>,
    mut food_writer: EventWriter<FoodEvent>,
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &mut SnakeHead)>,
    mut position: Query<&mut Position>,
    mut inputs: Query<&mut InputBuffer>,
) {
    if let Some((head_entity, mut head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .iter()
            .map(|e| *position.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        let mut head_pos = position.get_mut(head_entity).unwrap();

        if let Some(mut input_buffer) = inputs.iter_mut().next() {
            while let Some(input) = input_buffer.inputs.pop_front() {
                if input.opposite() != head.direction && input != head.direction {
                    head.direction = input;
                    break;
                }
            }
        }

        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        }

        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT
        {
            game_over_write.send(GameOverEvent);
            food_writer.send(FoodEvent);
        }

        if segment_positions.contains(&head_pos) {
            game_over_write.send(GameOverEvent);
            food_writer.send(FoodEvent);
        }

        segment_positions
            .iter()
            .zip(segments.iter().skip(1))
            .for_each(|(pos, segment)| {
                *position.get_mut(*segment).unwrap() = *pos;
            });
        *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));
    }
}

fn snake_movement_input(keyboard_input: Res<Input<KeyCode>>, mut inputs: Query<&mut InputBuffer>) {
    if let Some(mut input_buffer) = inputs.iter_mut().next() {
        if input_buffer.inputs.len() < 3 {
            if keyboard_input.just_pressed(KeyCode::Left) {
                input_buffer.inputs.push_back(Direction::Left)
            } else if keyboard_input.just_pressed(KeyCode::Down) {
                input_buffer.inputs.push_back(Direction::Down);
            } else if keyboard_input.just_pressed(KeyCode::Up) {
                input_buffer.inputs.push_back(Direction::Up);
            } else if keyboard_input.just_pressed(KeyCode::Right) {
                input_buffer.inputs.push_back(Direction::Right);
            }
        }
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    food_writer: EventWriter<FoodEvent>,
    segments_res: ResMut<SnakeSegments>,
    mut inputs: Query<&mut InputBuffer>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
) {
    if reader.iter().next().is_some() {
        for ent in food.iter().chain(segments.iter()) {
            commands.entity(ent).despawn();
        }

        setup_snake_game(commands, food_writer, segments_res);

        if let Some(mut input_buffer) = inputs.iter_mut().next() {
            input_buffer.inputs.clear();
        }
    }
}

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    mut food_writer: EventWriter<FoodEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.entity(ent).despawn();
                growth_writer.send(GrowthEvent);
                food_writer.send(FoodEvent);
            }
        }
    }
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
) {
    if growth_reader.iter().next().is_some() {
        segments.push(spawn_segment(commands, last_tail_position.0.unwrap()));
    }
}

fn size_scaling(windows: Res<Windows>, mut query: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut transform) in query.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
            1.0,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut query: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.0) + (tile_size / 2.0)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn food_spawner(
    commands: Commands,
    query: Query<&Position, With<SnakeSegment>>,
    mut food_reader: EventReader<FoodEvent>,
) {
    if food_reader.iter().next().is_some() {
        let mut new_position = Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
        };

        let mut collision = false;

        loop {
            for pos in query.iter() {
                if new_position == *pos {
                    collision = true;
                    break;
                }
            }

            if !collision {
                break;
            } else {
                new_position = Position {
                    x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
                    y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
                };
                collision = false;
            }
        }

        spawn_food(commands, new_position);
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Snake!".to_string(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            ..default()
        })
        .add_startup_system(setup_camera)
        .add_startup_system(setup_snake_game)
        .add_startup_system(init_inputs)
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .add_event::<GrowthEvent>()
        .add_system(snake_movement_input.before(snake_movement))
        .add_event::<GameOverEvent>()
        .add_event::<FoodEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.150))
                .with_system(snake_movement)
                .with_system(snake_eating.after(snake_movement))
                .with_system(snake_growth.after(snake_eating)),
        )
        .add_system(game_over.after(snake_movement))
        .add_system(food_spawner)
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        .add_plugins(DefaultPlugins)
        .run();
}
