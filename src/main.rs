#![feature(portable_simd)]
use bevy::prelude::*;

mod resources {
    use super::*;
    #[derive(Resource)]
    pub struct PlayerState {
        pub(crate) is_idle: bool,
        pub(crate) is_blinking: bool,
        pub(crate) heading: Heading,
    }
    #[derive(Resource, States, Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum PlayerAnimationState {
        #[default]
        Idle,
        Moving,
        Blinking,
    }
    #[derive(Resource)]
    pub struct PlayerAnimationIndices(pub animation::Indices);
    impl Default for PlayerAnimationIndices {
        fn default() -> Self {
            Self(animation::Indices { first: 0, last: 2 })
        }
    }
    impl PlayerAnimationIndices {
        pub fn update(&mut self, player_state: &PlayerState) {
            match (player_state.heading, player_state.is_idle) {
                (Heading::North, true) => self.0 = animation::Indices::from(0),
                (_, true) => self.0 = animation::Indices::from(0..2),
                (_, false) => self.0 = animation::Indices::from(0..9),
            }
        }
    }
    #[derive(Resource)]
    pub struct PlayerSpriteLayouts {
        north_idle: Handle<TextureAtlasLayout>,
        north_walk: Handle<TextureAtlasLayout>,
        east_idle: Handle<TextureAtlasLayout>,
        east_walk: Handle<TextureAtlasLayout>,
        south_idle: Handle<TextureAtlasLayout>,
        south_walk: Handle<TextureAtlasLayout>,
        west_idle: Handle<TextureAtlasLayout>,
        west_walk: Handle<TextureAtlasLayout>,
    }
    impl FromWorld for PlayerSpriteLayouts {
        fn from_world(world: &mut World) -> Self {
            let mut atlases = world
                .get_resource_mut::<Assets<TextureAtlasLayout>>()
                .unwrap();
            let north_idle = atlases.add(Player::LAYOUT(&Heading::North, true));
            let north_walk = atlases.add(Player::LAYOUT(&Heading::North, false));

            let east_idle = atlases.add(Player::LAYOUT(&Heading::East, true));
            let east_walk = atlases.add(Player::LAYOUT(&Heading::East, false));

            let south_idle = atlases.add(Player::LAYOUT(&Heading::South, true));
            let south_walk = atlases.add(Player::LAYOUT(&Heading::South, false));

            let west_idle = atlases.add(Player::LAYOUT(&Heading::West, true));
            let west_walk = atlases.add(Player::LAYOUT(&Heading::West, false));

            Self {
                north_idle,
                north_walk,
                east_idle,
                east_walk,
                south_idle,
                south_walk,
                west_idle,
                west_walk,
            }
        }
    }
    impl PlayerSpriteLayouts {
        pub fn resolve_from(&self, player_state: &PlayerState) -> Handle<TextureAtlasLayout> {
            match (player_state.heading, player_state.is_idle) {
                (Heading::North, true) => self.north_idle.clone_weak(),
                (Heading::North, false) => self.north_walk.clone_weak(),
                (Heading::East, true) => self.east_idle.clone_weak(),
                (Heading::East, false) => self.east_walk.clone_weak(),
                (Heading::South, true) => self.south_idle.clone_weak(),
                (Heading::South, false) => self.south_walk.clone_weak(),
                (Heading::West, true) => self.west_idle.clone_weak(),
                (Heading::West, false) => self.west_walk.clone_weak(),
            }
        }
    }
}
mod components {
    use super::*;
    use std::fmt;
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
    #[derive(Component, Copy, Clone)]
    #[repr(u8)]
    pub enum Heading {
        North,
        East,
        South,
        West,
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
        #[derive(Component)]
        pub struct Handler(pub(crate) for<'a> fn(&'a resources::PlayerState) -> Indices);
        impl Handler {
            pub fn for_link() -> Self {
                Self(
                    |player_state| match (player_state.heading, player_state.is_idle) {
                        (Heading::North, true) => animation::Indices::from(20),
                        (Heading::North, false) => animation::Indices::from(60..69),

                        (Heading::East, true) => animation::Indices::from(30..32),
                        (Heading::East, false) => animation::Indices::from(70..79),
                        (Heading::South, true) => animation::Indices::from(0..2),
                        (Heading::South, false) => animation::Indices::from(40..49),
                        (Heading::West, true) => animation::Indices::from(10..12),
                        (Heading::West, false) => animation::Indices::from(50..59),
                    },
                )
            }
        }
        #[derive(Component, Copy, Clone)]
        pub struct Indices {
            pub(crate) first: usize,
            pub(crate) last: usize,
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

mod systems {
    use super::*;
    pub mod sprite {
        use super::*;
        use std::time::Duration;
        pub fn spawn_link(
            mut commands: Commands,
            asset_server: Res<AssetServer>,
            mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        ) {
            let texture: Handle<Image> = asset_server.load("sprite_sheets/link.png");
            let texture_atlas_layout =
                TextureAtlasLayout::from_grid(Vec2::new(120.0, 130.0), 10, 8, None, None);
            let layout: Handle<TextureAtlasLayout> =
                texture_atlas_layouts.add(texture_atlas_layout);
            let heading = Heading::South;
            let animation_handler = animation::Handler::for_link();
            let player_state = resources::PlayerState {
                is_idle: true,
                is_blinking: false,
                heading: Heading::South,
            };
            let indices = animation_handler.0(&player_state);
            let index = indices.first;
            commands.spawn(Camera2dBundle::default());
            commands.insert_resource(player_state);
            commands.insert_resource(resources::PlayerAnimationIndices(indices));
            commands.spawn((
                Player,
                heading,
                animation_handler,
                SpriteSheetBundle {
                    texture,
                    transform: Transform::from_scale(Vec3::splat(0.5)),
                    atlas: TextureAtlas { layout, index },
                    ..Default::default()
                },
                animation::Timer(Timer::from_seconds(0.05, TimerMode::Repeating)),
            ));
        }
        pub fn update_layout(
            player_state: Res<resources::PlayerState>,
            player_sprite_layouts: Res<resources::PlayerSpriteLayouts>,
            mut query: Query<&mut TextureAtlas, With<Player>>,
        ) {
            if player_state.is_changed() {
                query.iter_mut().for_each(|mut texture_atlas| {
                    texture_atlas.layout = player_sprite_layouts.resolve_from(&player_state)
                });
            }
        }
        /// Should handle blinks during idle animation
        pub fn update_timer(
            player_state: Res<resources::PlayerState>,
            mut query: Query<&mut animation::Timer, With<Player>>,
        ) {
            if player_state.is_changed() {
                query.iter_mut().for_each(|mut timer| {
                    if player_state.is_blinking {
                        timer.set_duration(Duration::from_secs_f32(0.08))
                    } else if !player_state.is_idle {
                        timer.set_duration(Duration::from_secs_f32(0.03))
                    } else {
                        let nums = vec![0u8, 1u8, 2u8];
                        let rand = (std::ptr::addr_of!(nums) as u8) as f32 / u8::MAX as f32;
                        timer.set_duration(Duration::from_secs_f32(1.0 + rand))
                    }
                })
            }
        }

        pub fn update_player_sprite(
            mut player_state: ResMut<resources::PlayerState>,
            mut indices: ResMut<resources::PlayerAnimationIndices>,
            time: Res<Time>,
            mut query: Query<(&mut animation::Timer, &mut TextureAtlas), With<Player>>,
        ) {
            indices.update(&player_state);
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
                let current_blink_state = ((atlas.index % 10) != 0) && player_state.is_idle;
                if player_state.is_blinking != current_blink_state {
                    player_state.is_blinking = current_blink_state
                }
            });
            // query.iter_mut().for_each(|handler| {
            //     let new_indices = (handler.0)(&player_state);
            //     println!("new indices {} {}", new_indices.first, new_indices.last);
            //     indices.0.first = new_indices.first;
            //     indices.0.last = new_indices.last;
            // });
        }
        // pub fn animate(
        //     mut player_state: ResMut<resources::PlayerState>,
        //     indices: Res<resources::PlayerAnimationIndices>,
        //     time: Res<Time>,
        //     mut query: Query<(&mut animation::Timer, &mut TextureAtlas), With<Player>>,
        // ) {
        //     query.iter_mut().for_each(|(mut timer, mut atlas)| {
        //         timer.tick(time.delta());
        //         if timer.just_finished() {
        //             println!("atlas index: {}", atlas.index);
        //             atlas.index = if atlas.index >= indices.0.last {
        //                 indices.0.first
        //             } else {
        //                 atlas.index + 1
        //             };
        //             let current_blink_state = ((atlas.index % 10) != 0) && player_state.is_idle;
        //             if player_state.is_blinking != current_blink_state {
        //                 player_state.is_blinking = current_blink_state
        //             }
        //         }
        //     });
        // }
        // pub fn handle_idle(
        //     mut player_state: ResMut<resources::PlayerState>,
        //     keyboard_input: Res<ButtonInput<KeyCode>>,
        // ) {
        //     if keyboard_input.any_just_pressed(Player::INPUT_KEYS) {
        //         player_state.is_idle = false;
        //     } else {
        //         player_state.is_idle = true;
        //     }
        // }
        pub fn handle_heading(
            mut player_state: ResMut<resources::PlayerState>,
            keyboard_input: Res<ButtonInput<KeyCode>>,
        ) {
            // Handle heading only
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                player_state.is_idle = false;
                player_state.heading = Heading::North;
            };
            if keyboard_input.pressed(KeyCode::ArrowDown) {
                player_state.is_idle = false;
                player_state.heading = Heading::South;
            };
            if keyboard_input.pressed(KeyCode::ArrowRight) {
                player_state.is_idle = false;
                player_state.heading = Heading::East;
            };
            if keyboard_input.pressed(KeyCode::ArrowLeft) {
                player_state.is_idle = false;
                player_state.heading = Heading::West;
            };
            if !keyboard_input.any_pressed(Player::INPUT_KEYS) {
                player_state.is_idle = true;
            }
        }
        pub fn handle_movement(
            player_state: Res<resources::PlayerState>,
            time: Res<Time>,
            keyboard_input: Res<ButtonInput<KeyCode>>,
            mut query: Query<&mut Transform, With<Player>>,
        ) {
            query.iter_mut().for_each(|mut transform| {
                if !player_state.is_idle {
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
                    transform.translation +=
                        450. * (resultant_vector.normalize() * time.delta_seconds());
                }
            })
        }
        //pub fn update_indices(mut query: Query<&mut >)
        // pub fn setup(
        //     mut commands: Commands,
        //     asset_server: Res<AssetServer>,
        //     mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        // ) {
        //     let texture = asset_server.load("sprite_sheets/guy.png");
        //     let layout = TextureAtlasLayout::from_grid(Vec2::new(120.0, 130.0), 10, 8, None, None);
        //     let texture_atlas_layout = texture_atlas_layouts.add(layout);
        //     // Use only the subset of sprites in the sheet that make up the run animation (in 1D)
        //     let animation_indices = animation::Indices { first: 1, last: 4 };
        //     commands.spawn(Camera2dBundle::default());
        //     commands.spawn((
        //         SpriteSheetBundle {
        //             texture,
        //             atlas: TextureAtlas {
        //                 layout: texture_atlas_layout,
        //                 index: animation_indices.first,
        //             },
        //             transform: Transform::from_scale(Vec3::splat(1.0)),
        //             ..Default::default()
        //         },
        //         animation_indices,
        //         animation::Timer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        //     ));
        // }
    }
}
use systems::*;

mod plugins {
    use super::*;
    pub struct SpritePlugin;
    impl Plugin for SpritePlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
                .init_resource::<resources::PlayerSpriteLayouts>()
                .add_systems(Startup, sprite::spawn_link)
                .add_systems(
                    Update,
                    (
                        sprite::update_timer,
                        sprite::handle_heading,
                        sprite::handle_movement,
                        sprite::update_layout,
                        sprite::update_player_sprite,
                    )
                        .chain(),
                );
        }
    }
}

fn main() {
    App::new().add_plugins(plugins::SpritePlugin).run();
}
