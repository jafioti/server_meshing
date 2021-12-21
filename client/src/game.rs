use bevy::{prelude::*, app::AppExit};
use game_structs::Player;

#[derive(Component)]
pub struct CurrentPlayer {}

#[derive(Component)]
pub struct InterpolatePosition {
    pub target: Vec3,
}

/// set up a simple 3D scene
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player: Res<Player>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.0, 0.2, 1.0).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    }).insert(Player{id:player.id})
    .insert(CurrentPlayer{});
    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
        ..Default::default()
    });
}

pub fn move_block(mut player_query: Query<&mut Transform, With<Player>>, keys: Res<Input<KeyCode>>, time: Res<Time>) {
    let mut transform = player_query.iter_mut().next().unwrap();
    let mut velocity = Vec3::ZERO;

    for key in keys.get_pressed() {
        match key {
            KeyCode::W => velocity += Vec3::Z,
            KeyCode::S => velocity -= Vec3::Z,
            KeyCode::A => velocity += Vec3::X,
            KeyCode::D => velocity -= Vec3::X,
            _ => (),
        }
    }

    velocity = velocity.normalize();

    if !velocity.is_nan() {
        transform.translation += velocity * time.delta_seconds() * 10.;
    }
}

pub fn interpolate_positions(mut query: Query<(&mut Transform, &InterpolatePosition)>, time: Res<Time>) {
    for (mut transform, interpolate_position) in query.iter_mut() {
        if (interpolate_position.target - transform.translation).length() < time.delta_seconds() * 10. {
            transform.translation = interpolate_position.target;
        } else {
            let direction = (interpolate_position.target - transform.translation).normalize();
            transform.translation += direction * time.delta_seconds() * 10.;
        }
    }
}

pub fn exit_system(keys: Res<Input<KeyCode>>, player: Res<Player>, mut exit: EventWriter<AppExit>) {
    for key in keys.get_pressed() {
        if *key == KeyCode::Escape {
            crate::multiplayer::send_exit_to_server(player.id);
            exit.send(AppExit);
        }
    }
}