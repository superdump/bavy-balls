use std::time::Duration;

use bevy::{input::system::exit_on_esc_system, prelude::*, utils::Instant};
use bevy_rapier3d::{
    na::{Isometry3, Vector3},
    physics::TimestepMode,
    prelude::*,
};

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
    .add_system(exit_on_esc_system)
    .add_startup_system(setup_level)
    .add_system(spawn_balls)
    .add_system(despawn_balls);

    app.run();
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ground_scale = Vec3::new(100.0, 1.0, 100.0);
    let position = Isometry3::new(
        Vector3::new(0.0, 0.0, -50.0),
        Vector3::x() * 30.0f32.to_radians(),
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
        .insert(RigidBodyPositionSync::Discrete)
        .with_children(|builder| {
            builder
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(bevy::prelude::shape::Box::new(1.0, 1.0, 1.0))),
                    material: materials.add(StandardMaterial::from(Color::DARK_GRAY)),
                    transform: Transform::from_scale(ground_scale),
                    ..Default::default()
                })
                .insert_bundle(ColliderBundle {
                    shape: ColliderShape::cuboid(
                        0.5 * ground_scale.x,
                        0.5 * ground_scale.y,
                        0.5 * ground_scale.z,
                    )
                    .into(),
                    ..Default::default()
                })
                .insert(ColliderPositionSync::Discrete);
        });

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::default(),
        ..Default::default()
    });
}

#[derive(Component)]
struct LastSpawnTime {
    time: Instant,
}

impl Default for LastSpawnTime {
    fn default() -> Self {
        Self {
            time: Instant::now(),
        }
    }
}

#[derive(Component)]
struct Ball;

const MAX_BALLS: usize = 255;

fn spawn_balls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut last_spawn_time: Local<LastSpawnTime>,
    balls: Query<Entity, With<Ball>>,
) {
    let now = Instant::now();
    if balls.iter().count() < MAX_BALLS
        && (now > last_spawn_time.time + Duration::from_secs_f32(1.0)
            || keyboard_input.just_pressed(KeyCode::Space))
    {
        last_spawn_time.time = now;
        let spawn_point = Vec3::new(0.0, 100.0, -80.0);
        let ball_color = Color::AZURE;
        commands
            .spawn_bundle(RigidBodyBundle {
                body_type: RigidBodyType::Dynamic.into(),
                position: spawn_point.into(),
                ccd: RigidBodyCcd {
                    ccd_enabled: true,
                    ..Default::default()
                }
                .into(),
                ..Default::default()
            })
            .insert_bundle((Ball, RigidBodyPositionSync::Discrete))
            .with_children(|builder| {
                builder
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(bevy::prelude::shape::Icosphere {
                            radius: 1.0,
                            ..Default::default()
                        })),
                        material: materials.add(StandardMaterial {
                            base_color: ball_color,
                            emissive: ball_color * 10.0,
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
                            range: 10.0,
                            radius: 1.0,
                            shadows_enabled: false,
                            ..Default::default()
                        },
                        ..Default::default()
                    });
            });
    }
}

const MIN_Y: f32 = -100.0;

fn despawn_balls(mut commands: Commands, balls: Query<(Entity, &Transform), With<Ball>>) {
    for (ball, transform) in balls.iter() {
        if transform.translation.y < MIN_Y {
            commands.entity(ball).despawn_recursive();
        }
    }
}
