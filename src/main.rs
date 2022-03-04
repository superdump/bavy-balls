use std::time::Duration;

use bavy_balls::shapes::{mesh_to_collider_shape, HalfCylinderPath};
use bevy::{
    input::system::exit_on_esc_system, math::const_vec3, prelude::*, render::primitives::Aabb,
    ui::CAMERA_UI, utils::Instant,
};
use bevy_rapier3d::{
    na::{Isometry3, Vector3},
    physics::TimestepMode,
    prelude::*,
};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransform, LookTransformPlugin, Smoother,
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Menu,
    Playing,
    GameOver,
}

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "Bavy Balls".to_string(),
        width: 960.0,
        height: 540.0,
        resizable: false,
        ..Default::default()
    })
    .insert_resource(ClearColor(Color::BLACK))
    .add_plugins(DefaultPlugins)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .insert_resource(RapierConfiguration {
        timestep_mode: TimestepMode::InterpolatedTimestep,
        ..Default::default()
    })
    .add_plugin(LookTransformPlugin)
    .add_plugin(FpsCameraPlugin::default())
    .add_system(exit_on_esc_system);

    app.add_state(GameState::Menu)
        .insert_resource(RoundState {
            start: Instant::now(),
            players: Vec::new(),
        })
        .init_resource::<FollowMode>()
        .add_startup_system(setup)
        // .add_system(hacks)
        .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(setup_menu))
        .add_system_set(SystemSet::on_update(GameState::Menu).with_system(button_system))
        .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(cleanup_menu))
        .add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(setup_live_scoreboard)
                .with_system(setup_level)
                .with_system(start_round),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(follow_ball)
                .with_system(spawn_balls)
                .with_system(despawn_balls)
                .with_system(update_leaderboard),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::Playing)
                .with_system(despawn_level)
                .with_system(despawn_all_balls),
        )
        .add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(setup_game_over));

    app.run();
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<State<GameState>>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                state.set(GameState::Playing).ok();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

struct FontHandle {
    handle: Handle<Font>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(FontHandle {
        handle: asset_server.load("fonts/FiraSans-Bold.ttf"),
    });
}

fn setup_menu(mut commands: Commands, font_handle: Res<FontHandle>, mut windows: ResMut<Windows>) {
    for window in windows.iter_mut() {
        window.set_cursor_visibility(true);
    }
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|builder| {
            builder.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "BAVY BALLS",
                    TextStyle {
                        font: font_handle.handle.clone(),
                        font_size: 60.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                ),
                style: Style {
                    size: Size::new(Val::Px(300.0), Val::Px(65.0)),
                    // center button
                    margin: Rect::all(Val::Auto),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                ..Default::default()
            });
            builder
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        // center button
                        margin: Rect::all(Val::Auto),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "START",
                            TextStyle {
                                font: font_handle.handle.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                });
        });

    info!("Menu");
}

fn cleanup_menu(
    mut commands: Commands,
    cameras: Query<(Entity, &Camera)>,
    nodes: Query<Entity, With<Node>>,
) {
    for (entity, camera) in cameras.iter() {
        if camera.name == Some(CAMERA_UI.to_string()) {
            commands.entity(entity).despawn_recursive();
        }
    }
    for entity in nodes.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_game_over(mut state: ResMut<State<GameState>>) {
    info!("Game over!");
    state.set(GameState::Menu).ok();
}

// fn hacks(keyboard_input: Res<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
//     if keyboard_input.just_pressed(KeyCode::M) {
//         state.set(GameState::Menu).ok();
//     } else if keyboard_input.just_pressed(KeyCode::P) {
//         state.set(GameState::Playing).ok();
//     } else if keyboard_input.just_pressed(KeyCode::O) {
//         state.set(GameState::GameOver).ok();
//     }
// }

const SPAWN_POSITION: Vec3 = Vec3::ZERO;
const SPAWN_RADIUS: f32 = 75.0;

#[derive(Component)]
struct GameLevel;

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let half_cylinder_mesh = Mesh::from(HalfCylinderPath {
        start: SPAWN_POSITION,
        radius: SPAWN_RADIUS,
        segment_length: 100.0,
        n_segments: 10,
        seed: rand::random(),
        yaw_range: (-std::f32::consts::FRAC_PI_4)..std::f32::consts::FRAC_PI_4,
        pitch_range: (-std::f32::consts::FRAC_PI_4)..(-0.1 * std::f32::consts::FRAC_PI_4),
        ..Default::default()
    });
    let half_cylinder_collider = mesh_to_collider_shape(&half_cylinder_mesh)
        .expect("Failed to convert half cylinder mesh to collider");
    let half_cylinder_handle = meshes.add(half_cylinder_mesh);
    let mut half_cylinder_material = StandardMaterial::from(Color::SILVER);
    half_cylinder_material.perceptual_roughness = 0.5;
    let half_cylinder_material = materials.add(half_cylinder_material);

    spawn_halfpipe_segment(
        &mut commands,
        half_cylinder_handle,
        half_cylinder_material,
        half_cylinder_collider,
        Vec3::ZERO,
        Quat::IDENTITY,
    );

    commands
        .spawn_bundle(FpsCameraBundle::new(
            FpsCameraController {
                enabled: false,
                smoothing_weight: 0.99,
                ..Default::default()
            },
            PerspectiveCameraBundle::default(),
            SPAWN_POSITION + Vec3::new(0.0, 1.0, 1.0),
            SPAWN_POSITION,
        ))
        .insert(GameLevel);
}

fn spawn_halfpipe_segment(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    collider_shape: ColliderShape,
    translation: Vec3,
    rotation: Quat,
) {
    let (axis, angle) = rotation.to_axis_angle();
    let position = Isometry3::new(
        Vector3::new(translation.x, translation.y, translation.z),
        Vector3::new(axis.x, axis.y, axis.z) * angle,
    );
    commands
        .spawn_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Static.into(),
            position: RigidBodyPosition {
                position,
                next_position: position,
            }
            .into(),
            ..Default::default()
        })
        .insert_bundle((RigidBodyPositionSync::Discrete, GameLevel))
        .with_children(|builder| {
            builder
                .spawn_bundle(PbrBundle {
                    mesh,
                    material,
                    ..Default::default()
                })
                .insert_bundle(ColliderBundle {
                    shape: collider_shape.into(),
                    ..Default::default()
                })
                .insert_bundle((ColliderPositionSync::Discrete, Track));
        });
}

#[derive(Component)]
struct Track;

#[derive(Default)]
struct Prng {
    rng: Option<SmallRng>,
}

#[derive(Component)]
struct Ball;

const N_PLAYERS: usize = 10;

struct BallInfo {
    name: &'static str,
    color: Color,
}

const BALL_INFO: [BallInfo; N_PLAYERS] = [
    BallInfo {
        name: "RED",
        color: Color::RED,
    },
    BallInfo {
        name: "ORANGE",
        color: Color::ORANGE_RED,
    },
    BallInfo {
        name: "YELLOW",
        color: Color::ORANGE,
    },
    BallInfo {
        name: "GREEN",
        color: Color::GREEN,
    },
    BallInfo {
        name: "BLUE",
        color: Color::MIDNIGHT_BLUE,
    },
    BallInfo {
        name: "INDIGO",
        color: Color::BLUE,
    },
    BallInfo {
        name: "VIOLET",
        color: Color::INDIGO,
    },
    BallInfo {
        name: "WHITE",
        color: Color::WHITE,
    },
    BallInfo {
        name: "DARK_GRAY",
        color: Color::DARK_GRAY,
    },
    BallInfo {
        name: "BLACK",
        color: Color::BLACK,
    },
];

struct PlayerState {
    name: String,
    color: Color,
    entity: Option<Entity>,
    start: Instant,
    end: Option<Instant>,
    distance: f32,
    finished: bool,
}

impl PlayerState {
    fn new(name: String, color: Color, start: Instant) -> Self {
        Self {
            name,
            color,
            entity: None,
            start,
            end: None,
            distance: 0.0,
            finished: false,
        }
    }
}

struct RoundState {
    start: Instant,
    players: Vec<PlayerState>,
}

const MAX_DISADVANTAGE_MS: u64 = 10000;

fn start_round(mut rng: Local<Prng>, mut round: ResMut<RoundState>, mut windows: ResMut<Windows>) {
    for window in windows.iter_mut() {
        window.set_cursor_visibility(false);
    }
    if rng.rng.is_none() {
        rng.rng = Some(SmallRng::seed_from_u64(rand::random()));
    }
    let rng = rng.rng.as_mut().unwrap();
    round.start = Instant::now();
    round.players.clear();
    round.players = (0..N_PLAYERS)
        .map(|i| {
            PlayerState::new(
                format!("{} ({})", BALL_INFO[i].name, (i + 1) % N_PLAYERS),
                BALL_INFO[i].color,
                round.start + Duration::from_millis(rng.gen_range(0u64..MAX_DISADVANTAGE_MS)),
            )
        })
        .collect();
    info!("Starting the round!");
}

#[derive(Component)]
struct Leaderboard;

#[derive(Component)]
struct LeaderboardPlayer {
    index: usize,
}

#[derive(Component)]
struct LeaderboardPlayerName {
    index: usize,
}

fn setup_live_scoreboard(mut commands: Commands, font_handle: Res<FontHandle>) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());

    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            // right vertical fill
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::Center,
                        size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    color: Color::rgba(0.5, 0.5, 0.5, 0.15).into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    // Title
                    parent.spawn_bundle(TextBundle {
                        style: Style {
                            size: Size::new(Val::Undefined, Val::Px(25.)),
                            margin: Rect {
                                left: Val::Auto,
                                right: Val::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        text: Text::with_section(
                            "Leaderboard",
                            TextStyle {
                                font: font_handle.handle.clone(),
                                font_size: 25.,
                                color: Color::WHITE,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                    // List with hidden overflow
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::ColumnReverse,
                                align_self: AlignSelf::Center,
                                size: Size::new(Val::Percent(100.0), Val::Percent(50.0)),
                                overflow: Overflow::Hidden,
                                ..Default::default()
                            },
                            color: Color::rgba(0.75, 0.75, 0.75, 0.10).into(),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            // Moving panel
                            parent
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::ColumnReverse,
                                        flex_grow: 1.0,
                                        max_size: Size::new(Val::Undefined, Val::Undefined),
                                        ..Default::default()
                                    },
                                    color: Color::NONE.into(),
                                    ..Default::default()
                                })
                                .insert(Leaderboard)
                                .with_children(|parent| {
                                    // List items
                                    for (i, ball_info) in BALL_INFO.iter().enumerate() {
                                        parent
                                            .spawn_bundle(NodeBundle {
                                                style: Style {
                                                    justify_content: JustifyContent::FlexEnd,
                                                    size: Size::new(
                                                        Val::Px(200.0),
                                                        Val::Percent(100.0),
                                                    ),
                                                    flex_direction: FlexDirection::Row,
                                                    ..Default::default()
                                                },
                                                color: Color::NONE.into(),
                                                ..Default::default()
                                            })
                                            .with_children(|parent| {
                                                parent
                                                    .spawn_bundle(TextBundle {
                                                        style: Style {
                                                            flex_shrink: 0.,
                                                            size: Size::new(
                                                                Val::Undefined,
                                                                Val::Px(20.),
                                                            ),
                                                            margin: Rect {
                                                                left: Val::Px(10.),
                                                                right: Val::Auto,
                                                                ..Default::default()
                                                            },
                                                            ..Default::default()
                                                        },
                                                        text: Text::with_section(
                                                            ball_info.name,
                                                            TextStyle {
                                                                font: font_handle.handle.clone(),
                                                                font_size: 20.,
                                                                color: ball_info.color,
                                                            },
                                                            Default::default(),
                                                        ),
                                                        ..Default::default()
                                                    })
                                                    .insert(LeaderboardPlayerName { index: i });
                                                parent
                                                    .spawn_bundle(TextBundle {
                                                        style: Style {
                                                            flex_shrink: 0.,
                                                            size: Size::new(
                                                                Val::Undefined,
                                                                Val::Px(20.),
                                                            ),
                                                            margin: Rect {
                                                                right: Val::Px(10.),
                                                                left: Val::Auto,
                                                                ..Default::default()
                                                            },
                                                            ..Default::default()
                                                        },
                                                        text: Text::with_section(
                                                            ball_info.name,
                                                            TextStyle {
                                                                font: font_handle.handle.clone(),
                                                                font_size: 20.,
                                                                color: ball_info.color,
                                                            },
                                                            Default::default(),
                                                        ),
                                                        ..Default::default()
                                                    })
                                                    .insert(LeaderboardPlayer { index: i });
                                            });
                                    }
                                });
                        });
                });
        });
}

fn update_leaderboard(
    mut names: Query<(&LeaderboardPlayerName, &mut Text), Without<LeaderboardPlayer>>,
    mut distances: Query<(&LeaderboardPlayer, &mut Text), Without<LeaderboardPlayerName>>,
    round: Res<RoundState>,
) {
    let mut player_order = round
        .players
        .iter()
        .enumerate()
        .map(|(i, player)| (player.distance, player.end, i))
        .collect::<Vec<_>>();
    player_order.sort_unstable_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap()
            .then_with(|| a.1.unwrap_or(round.start).cmp(&b.1.unwrap_or(round.start)))
    });
    for (player, mut text) in distances.iter_mut() {
        let list_index = player.index;
        let (distance, end, player_index) = player_order[list_index];
        text.sections[0].value = if round.players[player_index].finished {
            format!("{:5.3}s", (end.unwrap() - round.start).as_secs_f64())
        } else {
            format!(
                "{}{:5.1}m",
                if end.is_some() && !round.players[player_index].finished {
                    "DNF "
                } else {
                    ""
                },
                distance.abs()
            )
        };
        text.sections[0].style.color = round.players[player_index].color;
    }
    for (player, mut text) in names.iter_mut() {
        let list_index = player.index;
        let (_, _, player_index) = player_order[list_index];
        text.sections[0].value = round.players[player_index].name.to_string();
        text.sections[0].style.color = round.players[player_index].color;
    }
}

fn spawn_balls(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut rng: Local<Prng>,
    mut round: ResMut<RoundState>,
) {
    let now = Instant::now();
    if rng.rng.is_none() {
        rng.rng = Some(SmallRng::seed_from_u64(rand::random()));
    }
    let rng = rng.rng.as_mut().unwrap();
    let meshes = meshes.into_inner();
    let materials = materials.into_inner();
    for player in round.players.iter_mut() {
        if player.entity.is_none() && player.end.is_none() && now > player.start {
            let spawn_point = SPAWN_POSITION
                + Vec3::new(
                    rng.gen_range((-0.9 * SPAWN_RADIUS + 1.0)..(0.9 * SPAWN_RADIUS - 1.0)),
                    0.0,
                    -1.0,
                );
            player.entity = Some(spawn_ball(
                &mut commands,
                meshes,
                materials,
                spawn_point,
                player.color,
            ));
        }
    }
}

fn spawn_ball(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    spawn_point: Vec3,
    ball_color: Color,
) -> Entity {
    commands
        .spawn_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Dynamic.into(),
            position: spawn_point.into(),
            velocity: RigidBodyVelocity {
                linvel: -1.0f32 * Vector3::z(),
                ..Default::default()
            }
            .into(),
            ccd: RigidBodyCcd {
                ccd_enabled: true,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        })
        .insert_bundle((
            Ball,
            RigidBodyPositionSync::Discrete,
            Transform::from_translation(spawn_point),
            GlobalTransform::from_translation(spawn_point),
        ))
        .with_children(|builder| {
            builder
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(bevy::prelude::shape::Icosphere {
                        radius: 1.0,
                        ..Default::default()
                    })),
                    material: materials.add(StandardMaterial {
                        base_color: ball_color,
                        emissive: ball_color,
                        perceptual_roughness: 0.9,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .insert_bundle(ColliderBundle {
                    shape: ColliderShape::ball(1.0).into(),
                    ..Default::default()
                })
                .insert(ColliderPositionSync::Discrete)
                .insert_bundle(PointLightBundle {
                    point_light: PointLight {
                        color: ball_color,
                        intensity: 5000.0,
                        range: 50.0,
                        radius: 1.0,
                        shadows_enabled: false,
                        ..Default::default()
                    },
                    ..Default::default()
                });
        })
        .id()
}

const BOUNDS: Vec3 = const_vec3!([0.0, -1000.0, f32::MIN]);
const BOUNDS_MARGIN: Vec3 = const_vec3!([0.0, -SPAWN_RADIUS - 10.0, 0.0]);

fn despawn_balls(
    mut commands: Commands,
    track: Query<&Aabb, With<Track>>,
    balls: Query<&GlobalTransform, With<Ball>>,
    mut bounds: Local<Option<Vec3>>,
    mut round: ResMut<RoundState>,
    mut state: ResMut<State<GameState>>,
) {
    *bounds = track
        .iter()
        .next()
        .map_or(Some(BOUNDS), |aabb| Some(aabb.min() + BOUNDS_MARGIN));
    let bounds = bounds.unwrap();
    let now = Instant::now();
    let round_start = round.start;
    let mut finished_count = 0;
    for player in round.players.iter_mut() {
        if let Some(entity) = player.entity {
            if let Ok(transform) = balls.get(entity) {
                player.distance = transform.translation.z.max(bounds.z);
                if transform.translation.y < bounds.y || transform.translation.z <= bounds.z {
                    player.end = Some(now);
                    let result = if transform.translation.z <= bounds.z {
                        player.finished = true;
                        "finished".to_string()
                    } else {
                        format!(
                            "did not finish ({:2.1}% complete)",
                            100.0 * player.distance / bounds.z
                        )
                    };
                    info!(
                        "{} {} in {:3.2}s ({:3.2}s)",
                        player.name,
                        result,
                        (now - round_start).as_secs_f32(),
                        (now - player.start).as_secs_f32()
                    );
                    commands.entity(entity).despawn_recursive();
                    player.entity = None;
                }
            }
        }
        if player.end.is_some() {
            finished_count += 1;
        }
    }
    if finished_count >= N_PLAYERS {
        state.set(GameState::GameOver).ok();
    }
}

fn despawn_level(mut commands: Commands, level_entities: Query<Entity, With<GameLevel>>) {
    for entity in level_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn despawn_all_balls(mut commands: Commands, mut round: ResMut<RoundState>) {
    for player in round.players.iter_mut() {
        if let Some(entity) = player.entity {
            commands.entity(entity).despawn_recursive();
            player.entity = None;
        }
    }
}

struct FollowMode {
    following: bool,
    index: usize,
    target: Option<Entity>,
}

impl Default for FollowMode {
    fn default() -> Self {
        Self {
            following: true,
            index: 0,
            target: None,
        }
    }
}

fn follow_ball(
    keyboard_input: Res<Input<KeyCode>>,
    mut follow_mode: ResMut<FollowMode>,
    balls: Query<(Entity, &GlobalTransform, &RigidBodyVelocityComponent), With<Ball>>,
    mut cameras: Query<(&mut FpsCameraController, &mut LookTransform, &mut Smoother)>,
    round: Res<RoundState>,
) {
    let (mut controller, mut look_transform, mut smoother) = cameras.single_mut();
    if keyboard_input.just_pressed(KeyCode::F) {
        follow_mode.following = !follow_mode.following;
        controller.enabled = !follow_mode.following;
        smoother.set_lag_weight(if follow_mode.following {
            0.99
        } else {
            controller.smoothing_weight
        });
    }
    if !follow_mode.following {
        return;
    }
    let mut updated = false;
    if keyboard_input.just_pressed(KeyCode::Key1) {
        follow_mode.index = 0;
        updated = true;
    } else if keyboard_input.just_pressed(KeyCode::Key2) {
        follow_mode.index = 1;
        updated = true;
    } else if keyboard_input.just_pressed(KeyCode::Key3) {
        follow_mode.index = 2;
        updated = true;
    } else if keyboard_input.just_pressed(KeyCode::Key4) {
        follow_mode.index = 3;
        updated = true;
    } else if keyboard_input.just_pressed(KeyCode::Key5) {
        follow_mode.index = 4;
        updated = true;
    } else if keyboard_input.just_pressed(KeyCode::Key6) {
        follow_mode.index = 5;
        updated = true;
    } else if keyboard_input.just_pressed(KeyCode::Key7) {
        follow_mode.index = 6;
        updated = true;
    } else if keyboard_input.just_pressed(KeyCode::Key8) {
        follow_mode.index = 7;
        updated = true;
    } else if keyboard_input.just_pressed(KeyCode::Key9) {
        follow_mode.index = 8;
        updated = true;
    } else if keyboard_input.just_pressed(KeyCode::Key0) {
        follow_mode.index = 9;
        updated = true;
    }
    follow_mode.target = round.players[follow_mode.index].entity;
    if updated {
        info!("Now following: {}", round.players[follow_mode.index].name);
    }
    if let Some(ball) = follow_mode.target {
        if let Ok((_, transform, velocity)) = balls.get(ball) {
            let linvel = Vec3::from_slice(velocity.linvel.as_slice()).normalize_or_zero();
            let right = linvel.cross(Vec3::Y);
            let up = right.cross(linvel);
            let offset = 100.0 * ((up - linvel) + 0.02 * Vec3::ONE);
            look_transform.target = transform.translation;
            look_transform.eye = transform.translation + offset;
        }
    }
}
