use bavy_balls::shapes::{mesh_to_collider_shape, HalfCylinder};
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
    LookTransformPlugin,
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
    .add_system(spawn_balls)
    .add_system(despawn_balls);

    app.run();
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let half_cylinder_mesh = Mesh::from(HalfCylinder::from_radius_and_length(25.0, 100.0));
    let half_cylinder_collider = mesh_to_collider_shape(&half_cylinder_mesh)
        .expect("Failed to convert half cylinder mesh to collider");
    let half_cylinder_handle = meshes.add(half_cylinder_mesh);

    // Ground
    let position = Isometry3::new(
        Vector3::new(0.0, 0.0, 0.0),
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
                    mesh: half_cylinder_handle,
                    material: materials.add(StandardMaterial::from(Color::WHITE)),
                    ..Default::default()
                })
                .insert_bundle(ColliderBundle {
                    shape: half_cylinder_collider.into(),
                    ..Default::default()
                })
                .insert(ColliderPositionSync::Discrete);
        });

    commands.spawn_bundle(FpsCameraBundle::new(
        FpsCameraController::default(),
        PerspectiveCameraBundle::default(),
        Vec3::new(0.0, 100.0, 100.0),
        Vec3::ZERO,
    ));
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
    keyboard_input: Res<Input<KeyCode>>,
    mut rng: Local<Prng>,
    mut to_spawn: Local<BallsToSpawn>,
    time: Res<Time>,
    balls: Query<Entity, With<Ball>>,
) {
    let t = time.seconds_since_startup();
    let ball_count = balls.iter().count();
    if (ball_count == 0 && to_spawn.balls.is_empty()) || keyboard_input.just_pressed(KeyCode::Space)
    {
        if rng.rng.is_none() {
            rng.rng = Some(SmallRng::seed_from_u64(1234));
        }
        let rng = rng.rng.as_mut().unwrap();
        for _ in 0..N_PLAYERS {
            let spawn_time = t + rng.gen_range(0.0..10.0);
            let spawn_point = Vec3::new(rng.gen_range(-50.0..50.0), 100.0, -30.0);
            let ball_color = Color::rgb(rng.gen(), rng.gen(), rng.gen());
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
                        range: 50.0,
                        radius: 1.0,
                        shadows_enabled: false,
                        ..Default::default()
                    },
                    ..Default::default()
                });
        });
}

const MIN_Y: f32 = -100.0;

fn despawn_balls(mut commands: Commands, balls: Query<(Entity, &Transform), With<Ball>>) {
    for (ball, transform) in balls.iter() {
        if transform.translation.y < MIN_Y {
            commands.entity(ball).despawn_recursive();
        }
    }
}
