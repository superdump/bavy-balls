use bavy_balls::shapes::{mesh_to_collider_shape, HalfCylinderPath};
use bevy::{input::system::exit_on_esc_system, prelude::*};
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
    .add_system(exit_on_esc_system)
    .add_startup_system(setup_level)
    .init_resource::<FollowMode>()
    .add_system(follow_ball)
    .add_system(spawn_balls)
    .add_system(despawn_balls);

    app.run();
}

const SPAWN_POSITION: Vec3 = Vec3::ZERO;
const SPAWN_RADIUS: f32 = 75.0;

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let half_cylinder_mesh = Mesh::from(HalfCylinderPath {
        start: SPAWN_POSITION,
        radius: SPAWN_RADIUS,
        segment_length: 100.0,
        n_segments: 100,
        seed: 4321,
        yaw_range: (-std::f32::consts::FRAC_PI_4)..std::f32::consts::FRAC_PI_4,
        pitch_range: (-std::f32::consts::FRAC_PI_4)..(-0.1 * std::f32::consts::FRAC_PI_4),
        ..Default::default()
    });
    let half_cylinder_collider = mesh_to_collider_shape(&half_cylinder_mesh)
        .expect("Failed to convert half cylinder mesh to collider");
    let half_cylinder_handle = meshes.add(half_cylinder_mesh);
    let half_cylinder_material = materials.add(StandardMaterial::from(Color::WHITE));

    spawn_halfpipe_segment(
        &mut commands,
        half_cylinder_handle,
        half_cylinder_material,
        half_cylinder_collider,
        Vec3::ZERO,
        Quat::IDENTITY,
    );

    commands.spawn_bundle(FpsCameraBundle::new(
        FpsCameraController::default(),
        PerspectiveCameraBundle::default(),
        SPAWN_POSITION + Vec3::new(0.0, 1.0, 1.0),
        SPAWN_POSITION,
    ));
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
        .insert(RigidBodyPositionSync::Discrete)
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
                .insert(ColliderPositionSync::Discrete);
        });
}

#[derive(Default)]
struct Prng {
    rng: Option<SmallRng>,
}

#[derive(Component)]
struct Ball;

#[derive(Default)]
struct BallsToSpawn {
    balls: Vec<(f64, Vec3, Color)>,
}

const N_PLAYERS: usize = 10;

#[allow(clippy::too_many_arguments)]
fn spawn_balls(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut rng: Local<Prng>,
    mut to_spawn: Local<BallsToSpawn>,
    time: Res<Time>,
    balls: Query<Entity, With<Ball>>,
) {
    let t = time.seconds_since_startup();
    let ball_count = balls.iter().count();
    if ball_count == 0 && to_spawn.balls.is_empty() {
        if rng.rng.is_none() {
            rng.rng = Some(SmallRng::seed_from_u64(1234));
        }
        let rng = rng.rng.as_mut().unwrap();
        for i in 0..N_PLAYERS {
            let spawn_time = t + rng.gen_range(0.0..10.0);
            let spawn_point = Vec3::new(
                rng.gen_range((-0.9 * SPAWN_RADIUS + 1.0)..(0.9 * SPAWN_RADIUS - 1.0)),
                0.0,
                -1.0,
            );
            let ball_color = Color::hsl(360.0 * (i as f32) / (N_PLAYERS as f32), 1.0, 0.5);
            to_spawn.balls.push((spawn_time, spawn_point, ball_color));
        }
    }
    let meshes = meshes.into_inner();
    let materials = materials.into_inner();
    for i in (0..to_spawn.balls.len()).rev() {
        if t > to_spawn.balls[i].0 {
            let (_t, spawn_point, ball_color) = to_spawn.balls.swap_remove(i);
            spawn_ball(&mut commands, meshes, materials, spawn_point, ball_color);
        }
    }
}

fn spawn_ball(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    spawn_point: Vec3,
    ball_color: Color,
) {
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
        });
}

const MIN_Y: f32 = -1000.0;

fn despawn_balls(mut commands: Commands, balls: Query<(Entity, &Transform), With<Ball>>) {
    for (ball, transform) in balls.iter() {
        if transform.translation.y < MIN_Y {
            commands.entity(ball).despawn_recursive();
        }
    }
}

#[derive(Default)]
struct FollowMode {
    target: Option<Entity>,
}

fn follow_ball(
    keyboard_input: Res<Input<KeyCode>>,
    mut follow_mode: ResMut<FollowMode>,
    balls: Query<(Entity, &GlobalTransform, &RigidBodyVelocityComponent), With<Ball>>,
    mut cameras: Query<(&FpsCameraController, &mut LookTransform, &mut Smoother)>,
) {
    let (controller, mut look_transform, mut smoother) = cameras.single_mut();
    let n_balls = balls.iter().count();
    if keyboard_input.just_pressed(KeyCode::F) {
        follow_mode.target = match follow_mode.target {
            Some(_) => {
                smoother.set_lag_weight(controller.smoothing_weight);
                None
            }
            None => {
                smoother.set_lag_weight(0.99);
                Some(balls.iter().nth(n_balls / 2).unwrap().0)
            }
        };
    }
    let mut new_ball = None;
    if let Some(ball) = follow_mode.target {
        if let Some((_, transform, velocity)) = balls.get(ball).ok().or_else(|| {
            new_ball = balls.iter().nth(n_balls / 2);
            new_ball
        }) {
            let linvel = Vec3::from_slice(velocity.linvel.as_slice()).normalize_or_zero();
            let right = linvel.cross(Vec3::Y);
            let up = right.cross(linvel);
            let offset = 100.0 * ((up - linvel) + 0.02 * Vec3::ONE);
            look_transform.target = transform.translation;
            look_transform.eye = transform.translation + offset;
        }
    }
    if let Some((new_ball, ..)) = new_ball {
        follow_mode.target = Some(new_ball);
    }
}
