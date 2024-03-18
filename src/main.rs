#![feature(portable_simd)]
use bevy::prelude::*;

mod player {
    //! Player-related stuff
    use super::*;
    mod resources {
        //! Player resources
        use super::*;
        use std::ops::{Deref, DerefMut};
        #[derive(Resource)]
        pub struct AnimationIndices(pub animation::Indices);
        impl Default for AnimationIndices {
            fn default() -> Self {
                Self(animation::Indices { first: 0, last: 2 })
            }
        }
        impl Deref for AnimationIndices {
            type Target = animation::Indices;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl DerefMut for AnimationIndices {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
        #[derive(Resource)]
        pub struct SpriteLayouts {
            north_idle: Handle<TextureAtlasLayout>,
            north_walk: Handle<TextureAtlasLayout>,
            east_idle: Handle<TextureAtlasLayout>,
            east_walk: Handle<TextureAtlasLayout>,
            south_idle: Handle<TextureAtlasLayout>,
            south_walk: Handle<TextureAtlasLayout>,
            west_idle: Handle<TextureAtlasLayout>,
            west_walk: Handle<TextureAtlasLayout>,
        }
        impl FromWorld for SpriteLayouts {
            fn from_world(world: &mut World) -> Self {
                let mut atlases = world
                    .get_resource_mut::<Assets<TextureAtlasLayout>>()
                    .unwrap();
                Self {
                    north_idle: atlases.add(Player::LAYOUT(&Heading::North, true)),
                    north_walk: atlases.add(Player::LAYOUT(&Heading::North, false)),
                    east_idle: atlases.add(Player::LAYOUT(&Heading::East, true)),
                    east_walk: atlases.add(Player::LAYOUT(&Heading::East, false)),
                    south_idle: atlases.add(Player::LAYOUT(&Heading::South, true)),
                    south_walk: atlases.add(Player::LAYOUT(&Heading::South, false)),
                    west_idle: atlases.add(Player::LAYOUT(&Heading::West, true)),
                    west_walk: atlases.add(Player::LAYOUT(&Heading::West, false)),
                }
            }
        }
        impl SpriteLayouts {
            pub fn resolve(
                &self,
                heading: &Heading,
                animation_state: &animation::State,
            ) -> Handle<TextureAtlasLayout> {
                match (heading, animation_state) {
                    (Heading::North, animation::State::Idle) => self.north_idle.clone_weak(),
                    (Heading::North, animation::State::Active) => self.north_walk.clone_weak(),
                    (Heading::East, animation::State::Idle) => self.east_idle.clone_weak(),
                    (Heading::East, animation::State::Active) => self.east_walk.clone_weak(),
                    (Heading::South, animation::State::Idle) => self.south_idle.clone_weak(),
                    (Heading::South, animation::State::Active) => self.south_walk.clone_weak(),
                    (Heading::West, animation::State::Idle) => self.west_idle.clone_weak(),
                    (Heading::West, animation::State::Active) => self.west_walk.clone_weak(),
                }
            }
        }
    }
    pub(super) mod systems {
        //! Player systems
        use super::*;
        pub fn spawn(
            mut commands: Commands,
            asset_server: Res<AssetServer>,
            mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        ) {
            println!("Spawning link");
            // One bad init I guess (actual layouts will be handled runtime)
            let texture: Handle<Image> = asset_server.load("sprite_sheets/link.png");
            let texture_atlas_layout =
                TextureAtlasLayout::from_grid(Vec2::new(120.0, 130.0), 10, 8, None, None);
            let layout: Handle<TextureAtlasLayout> =
                texture_atlas_layouts.add(texture_atlas_layout);

            let animation_indices = resources::AnimationIndices::default();
            let index = animation_indices.first;
            commands.insert_resource(animation_indices);
            commands.spawn(Camera2dBundle::default());
            commands.spawn((
                Player,
                components::animation::State::Idle,
                components::animation::Blinking(false),
                components::Heading::South,
                SpriteSheetBundle {
                    texture,
                    transform: Transform::from_scale(Vec3::splat(0.5)),
                    atlas: TextureAtlas { layout, index },
                    ..Default::default()
                },
                components::animation::Timer(Timer::from_seconds(0.05, TimerMode::Repeating)),
            ));
        }
        pub fn input_handler(
            time: Res<Time>,
            keyboard_input: Res<ButtonInput<KeyCode>>,
            mut query: Query<(&mut Heading, &mut animation::State, &mut Transform), With<Player>>,
        ) {
            if keyboard_input.any_pressed(Player::INPUT_KEYS) {
                query
                    .iter_mut()
                    .for_each(|(mut heading, mut state, mut transform)| {
                        state.set_if_neq(animation::State::Active);
                        let mut resultant_vector = Vec3::new(0., 0., 0.);
                        // Exclusive, prevents NaN resultant_vector
                        if keyboard_input.pressed(KeyCode::ArrowUp) {
                            resultant_vector.y += 1.;
                        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
                            resultant_vector.y -= 1.;
                        };
                        if keyboard_input.pressed(KeyCode::ArrowLeft) {
                            resultant_vector.x -= 1.;
                        } else if keyboard_input.pressed(KeyCode::ArrowRight) {
                            resultant_vector.x += 1.;
                        };
                        // Update transform based on resultant vector
                        transform.translation +=
                            450. * (resultant_vector.normalize() * time.delta_seconds());
                        // Additionally, match the heading based on resultant vector's angle
                        heading.set_if_neq(resultant_vector.into());
                    });
            } else if !keyboard_input.any_pressed(Player::INPUT_KEYS) {
                // Here we can say the player is definitely idle
                query.iter_mut().for_each(|(_, mut animation_state, _)| {
                    animation_state.set_if_neq(animation::State::Idle);
                });
            }
        }
        pub fn animate_sprite(
            indices: Res<resources::AnimationIndices>,
            time: Res<Time>,
            mut query: Query<(&mut animation::Timer, &mut TextureAtlas), With<Player>>,
        ) {
            query.iter_mut().for_each(|(mut timer, mut atlas)| {
                // Maybe this will solve it?
                if atlas.index >= indices.0.last {
                    atlas.index = indices.0.first
                };
                timer.tick(time.delta());
                if timer.just_finished() {
                    atlas.index = if atlas.index >= indices.0.last {
                        indices.0.first
                    } else {
                        atlas.index + 1
                    }
                }
            });
            // query.iter_mut().for_each(|handler| {
            //     let new_indices = (handler.0)(&player_state);
            //     println!("new indices {} {}", new_indices.first, new_indices.last);
            //     indices.0.first = new_indices.first;
            //     indices.0.last = new_indices.last;
            // });
        }
        pub mod update {
            //! Systems related to updating player components and resources
            use super::*;
            use std::time::Duration;
            fn animation_timer(
                mut query: Query<
                    (
                        &mut animation::Timer,
                        &animation::State,
                        &animation::Blinking,
                    ),
                    (
                        With<Player>,
                        Or<(Changed<animation::Blinking>, Changed<animation::State>)>,
                    ),
                >,
            ) {
                query
                    .iter_mut()
                    .for_each(|(mut timer, state, is_blinking)| {
                        if is_blinking.0 {
                            timer.set_duration(Duration::from_secs_f32(0.08))
                        } else if state == &animation::State::Active {
                            timer.set_duration(Duration::from_secs_f32(0.03))
                        } else {
                            let nums = vec![0u8, 1u8, 2u8];
                            let rand = (std::ptr::addr_of!(nums) as u8) as f32 / u8::MAX as f32;
                            timer.set_duration(Duration::from_secs_f32(1.0 + rand))
                        }
                    })
            }
            fn animation_state(
                mut query: Query<
                    (
                        &mut animation::Blinking,
                        &TextureAtlas,
                        &animation::State,
                        &Heading,
                    ),
                    (With<Player>, Changed<TextureAtlas>),
                >,
            ) {
                query
                    .iter_mut()
                    .for_each(|(mut blinking, texture_atlas, state, heading)| {
                        if heading != &Heading::North && state == &animation::State::Idle {
                            blinking.set_if_neq(animation::Blinking(texture_atlas.index != 0));
                        } else {
                            blinking.set_if_neq(animation::Blinking(
                                state != &animation::State::Active,
                            ));
                        }
                    });
            }
            fn animation_indices(
                mut animation_indices: ResMut<resources::AnimationIndices>, // Access indices mutably
                query: Query<
                    (&Heading, &animation::State), // Get the heading and animation state
                    (
                        With<Player>,                                      // Belonging to the player
                        Or<(Changed<Heading>, Changed<animation::State>)>, // where either has changed
                    ),
                >,
            ) {
                query
                    .iter()
                    .for_each(|(heading, state)| match (heading, state) {
                        (Heading::North, animation::State::Idle) => {
                            animation_indices.0 = animation::Indices::from(0)
                        }
                        (_, animation::State::Idle) => {
                            animation_indices.0 = animation::Indices::from(0..2)
                        }
                        (_, animation::State::Active) => {
                            animation_indices.0 = animation::Indices::from(0..9)
                        }
                    });
            }
            fn sprite_layout(
                sprite_layouts: Res<resources::SpriteLayouts>,
                mut query: Query<
                    (&mut TextureAtlas, &Heading, &animation::State),
                    (
                        With<Player>,
                        Or<(Changed<Heading>, Changed<animation::State>)>,
                    ),
                >,
            ) {
                query
                    .iter_mut()
                    .for_each(|(mut texture_atlas, heading, animation_state)| {
                        texture_atlas.layout = sprite_layouts.resolve(heading, animation_state)
                    });
            }
            pub(super) struct UpdatePlayer;
            impl Plugin for UpdatePlayer {
                fn build(&self, app: &mut App) {
                    app.add_systems(
                        Update,
                        (
                            systems::input_handler,
                            systems::update::animation_state,
                            systems::update::animation_timer,
                            systems::update::animation_indices,
                            systems::update::sprite_layout,
                            systems::animate_sprite,
                        )
                            .chain(),
                    );
                }
            }
        }
        pub struct PlayerPlugin;
        impl Plugin for PlayerPlugin {
            fn build(&self, app: &mut App) {
                app.init_resource::<resources::SpriteLayouts>()
                    .add_systems(Startup, systems::spawn)
                    .add_plugins(systems::update::UpdatePlayer);
            }
        }
    }
}
mod components {
    use super::*;
    use std::fmt;
    /// Marker for player character
    #[derive(Component)]
    pub struct Player;
    impl Player {
        pub const INPUT_KEYS: [KeyCode; 4] = [
            KeyCode::ArrowUp,
            KeyCode::ArrowRight,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
        ];
        pub const SHEET_SIZE: (f32, f32) = (1200.0, 1040.0);
        pub const FULL_COLUMN_COUNT: usize = 10;
        pub const COLUMN_SIZE: f32 = Self::SHEET_SIZE.0 / Self::FULL_COLUMN_COUNT as f32;
        pub const COLUMN_COUNT: for<'a> fn(&'a Heading, bool) -> usize =
            |heading, is_idle| match (heading, is_idle) {
                (Heading::North, true) => 1,
                (_, true) => 3,
                (_, false) => Self::FULL_COLUMN_COUNT,
            };
        pub const ROW_COUNT: usize = 8;
        pub const ROW_SIZE: f32 = Self::SHEET_SIZE.1 / Self::ROW_COUNT as f32;
        pub const TILE_SIZE: Vec2 = Vec2::new(Self::COLUMN_SIZE, Self::ROW_SIZE);
        pub const UNIT_OFFSET: Vec2 = Vec2::new(0.0, Self::ROW_SIZE);
        pub const OFFSET_FACTOR: for<'a> fn(&'a Heading, bool) -> f32 =
            |heading, is_idle| match (heading, is_idle) {
                (Heading::South, true) => 0.0,
                (Heading::West, true) => 1.0,
                (Heading::North, true) => 2.0,
                (Heading::East, true) => 3.0,
                (Heading::South, false) => 4.0,
                (Heading::West, false) => 5.0,
                (Heading::North, false) => 6.0,
                (Heading::East, false) => 7.0,
            };
        pub const OFFSET: for<'a> fn(&'a Heading, bool) -> Option<Vec2> =
            |heading, is_idle| match (heading, is_idle) {
                (Heading::South, true) => None,
                _ => Some(Self::OFFSET_FACTOR(&heading, is_idle) * Self::UNIT_OFFSET),
            };
        pub const LAYOUT: for<'a> fn(&'a Heading, bool) -> TextureAtlasLayout =
            |heading, is_idle| match (heading, is_idle) {
                _ => TextureAtlasLayout::from_grid(
                    Self::TILE_SIZE,
                    Self::COLUMN_COUNT(heading, is_idle),
                    1,
                    None,
                    Self::OFFSET(heading, is_idle),
                ),
            };
    }
    #[derive(Component, Copy, Clone, PartialEq)]
    #[repr(u8)]
    pub enum Heading {
        North,
        East,
        South,
        West,
    }
    #[allow(dead_code)]
    impl Heading {
        const NORTH: Vec3 = Vec3::new(0., 1., 0.);
        const EAST: Vec3 = Vec3::new(1., 0., 0.);
        const SOUTH: Vec3 = Vec3::new(0., -1., 0.);
        const WEST: Vec3 = Vec3::new(-1., 0., 0.);
    }
    impl From<Vec3> for Heading {
        #[inline]
        fn from(value: Vec3) -> Self {
            if value.x > 0. {
                // Right half
                let angle_to_east = value.angle_between(Heading::EAST);
                if value.y > 0. && angle_to_east > (std::f32::consts::PI / 4.) {
                    Heading::North
                } else if value.y < 0. && angle_to_east > (std::f32::consts::PI / 4.) {
                    Heading::South
                } else {
                    Heading::East
                }
            } else {
                // Left half
                let angle_to_west = value.angle_between(Heading::WEST);
                if value.y > 0. && angle_to_west > (std::f32::consts::PI / 4.) {
                    Heading::North
                } else if value.y < 0. && angle_to_west > (std::f32::consts::PI / 4.) {
                    Heading::South
                } else {
                    Heading::West
                }
            }
        }
    }
    impl From<&Vec3> for Heading {
        #[inline]
        fn from(value: &Vec3) -> Self {
            if value.x > 0. {
                // Right half
                let angle_to_east = value.angle_between(Heading::EAST);
                if value.y > 0. && angle_to_east > (std::f32::consts::PI / 4.) {
                    Heading::North
                } else if value.y < 0. && angle_to_east > (std::f32::consts::PI / 4.) {
                    Heading::South
                } else {
                    Heading::East
                }
            } else {
                // Left half
                let angle_to_west = value.angle_between(Heading::WEST);
                if value.y > 0. && angle_to_west > (std::f32::consts::PI / 4.) {
                    Heading::North
                } else if value.y < 0. && angle_to_west > (std::f32::consts::PI / 4.) {
                    Heading::South
                } else {
                    Heading::West
                }
            }
        }
    }
    #[derive(Component)]
    pub struct Position {
        x: f32,
        y: f32,
    }
    impl fmt::Display for Position {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(format_args!("x: {}, y: {}", self.x, self.y))
        }
    }
    pub mod animation {
        use super::*;
        #[derive(Component, Copy, Clone, Default, PartialEq)]
        #[repr(u8)]
        pub enum State {
            #[default]
            Idle,
            Active,
        }
        #[derive(Component, Copy, Clone, Default, PartialEq)]
        pub struct Blinking(pub bool);
        #[derive(Component, Copy, Clone)]
        pub struct Indices {
            pub(crate) first: usize,
            pub(crate) last: usize,
        }
        impl Default for Indices {
            fn default() -> Self {
                Self { first: 0, last: 0 }
            }
        }
        impl From<usize> for Indices {
            fn from(value: usize) -> Self {
                Self {
                    first: value,
                    last: value,
                }
            }
        }
        impl From<std::ops::Range<usize>> for Indices {
            fn from(value: std::ops::Range<usize>) -> Self {
                Self {
                    first: value.start,
                    last: value.end,
                }
            }
        }
        #[derive(Component, Deref, DerefMut)]
        pub struct Timer(pub(crate) bevy::prelude::Timer);
        impl Timer {}
    }
}
use components::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            crate::player::systems::PlayerPlugin,
        ))
        .run();
}
