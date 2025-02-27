use std::path::Path;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::mouse::MouseWheel,
    math::Quat,
    prelude::*,
    render::camera::{Camera, OrthographicProjection},
    window::{WindowId, WindowResized},
};

use rand::Rng;

struct PrintTimer(Timer);
struct Position(Transform);
enum Direction {
    Clockwise,
    CounterClockwise,
}
struct RotationRate(f32);

const CAMERA_SPEED: f32 = 10.0;
const SCALE_FACTOR: f32 = 0.025;

fn main() {
    App::build()
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::DEBUG,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(tick.system().label("Tick"))
        .add_system(rotate_entity.system().after("Tick").label("Game"))
        .add_system(move_camera.system().after("Game"))
        .add_system(zoom_camera.system().after("Game"))
        .run()
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();

    let tile_size = Vec2::splat(64.0);
    let map_size = Vec2::splat(320.0);

    let half_x = (map_size.x / 32.0) as i32;
    let half_y = (map_size.y / 32.0) as i32;

    let sprite_path = Path::new("branding").join("icon.png");
    let sprite_handle = materials.add(assets.load(sprite_path).into());

    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d())
        .insert(PrintTimer(Timer::from_seconds(1.0, true)))
        .insert(Position(Transform::from_translation(Vec3::new(
            0.0, 0.0, 1000.0,
        ))));

    for y in -half_y..half_y {
        for x in -half_x..half_x {
            let position = Vec2::new(x as f32, y as f32);
            let translation = (position * tile_size).extend(rng.gen::<f32>());
            let rotation = Quat::from_rotation_z(rng.gen::<f32>());
            let scale = Vec3::splat(rng.gen::<f32>() * 2.0);

            commands
                .spawn()
                .insert_bundle(SpriteBundle {
                    material: sprite_handle.clone(),
                    transform: Transform {
                        translation,
                        rotation,
                        scale,
                    },
                    sprite: Sprite::new(tile_size),
                    ..Default::default()
                })
                .insert(if rng.gen::<f32>() > 0.5 {
                    Direction::CounterClockwise
                } else {
                    Direction::Clockwise
                })
                .insert(RotationRate(rng.gen::<f32>() * 5.0));
        }
    }
}

fn rotate_entity(time: Res<Time>, mut query: Query<(&mut Transform, &Direction, &RotationRate)>) {
    for (mut transform, direction, rate) in query.iter_mut() {
        let rotation_direction = match *direction {
            Direction::Clockwise => 1.0 as f32,
            Direction::CounterClockwise => -1.0 as f32,
        };
        transform.rotation *=
            Quat::from_rotation_z(time.delta_seconds() * rotation_direction * rate.0);
    }
}

fn move_camera(time: Res<Time>, mut query: Query<(&mut Transform, &mut Position), With<Camera>>) {
    for (mut transform, mut position) in query.iter_mut() {
        position
            .0
            .rotate(Quat::from_rotation_z(time.delta_seconds() * 0.5));
        position.0 =
            position.0 * Transform::from_translation(Vec3::X * CAMERA_SPEED * time.delta_seconds());
        transform.translation = position.0.translation;
        transform.rotation *= Quat::from_rotation_z(time.delta_seconds() / 2.0);
    }
}

fn zoom_camera(
    windows: Res<Windows>,
    mut projection_query: Query<&mut OrthographicProjection>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut window_resized: EventWriter<WindowResized>,
) {
    for event in mouse_wheel_events.iter() {
        for mut projection in projection_query.iter_mut() {
            projection.scale = (projection.scale - event.y * SCALE_FACTOR)
                .max(0.1)
                .min(1.0);

            let window = windows.get_primary().unwrap();
            window_resized.send(WindowResized {
                id: WindowId::primary(),
                width: window.width(),
                height: window.height(),
            });
        }
    }
}

fn tick(time: Res<Time>, sprites: Query<&Sprite>, mut query: Query<&mut PrintTimer>) {
    for mut timer in query.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            println!("Sprites: {}", sprites.iter().count());
        }
    }
}
