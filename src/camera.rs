use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::window::PrimaryWindow;
use bevy::prelude::*;
use EulerRot::YXZ;


// Camera properties
#[derive(Component)]
pub struct CameraSettings {
    pub speed: f32,
    pub sensitivity: f32,
}


/// Spawns camera.
pub fn spawn_camera(mut commands: Commands) {
    let start_position = Vec3::new(0.0, 18.0, 0.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(start_position),
        GlobalTransform::default(),
        CameraSettings {
            speed: 50.0,
            sensitivity: 0.4,
        },
    ));
}


/// Handles camera looking movement.
pub fn camera_look(
    camera: Single<(&mut Transform, &CameraSettings)>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if !window.focused {
        return;
    }
    let (mut transform, camera_data) = camera.into_inner();
    let sensitivity = 
        camera_data.sensitivity * (window.width().min(window.height()) / window.height());

    let deltatime = time.delta_secs();
    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(YXZ);
    pitch -= mouse_motion.delta.y * deltatime * sensitivity;
    pitch = pitch.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    yaw -= mouse_motion.delta.x * deltatime * sensitivity;
    transform.rotation = Quat::from_euler(YXZ, yaw, pitch, 0.0);
}



/// Handles camera movement and defines movement keybinds.
pub fn camera_movement(
    mut query: Query<(&mut Transform, &CameraSettings)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    for (mut transform, settings) in &mut query {
        let mut delta = Vec3::ZERO;
        let mut speed = settings.speed;

        if input.pressed(KeyCode::KeyW) { delta.z -= 1.0; }
        if input.pressed(KeyCode::KeyS) { delta.z += 1.0; }
        if input.pressed(KeyCode::KeyA) { delta.x -= 1.0; }
        if input.pressed(KeyCode::KeyD) { delta.x += 1.0; }
        if input.pressed(KeyCode::KeyQ) { delta.y -= 1.0; }
        if input.pressed(KeyCode::KeyE) { delta.y += 1.0; }
        if input.pressed(KeyCode::ShiftLeft) { speed *= 2.0; }

        let (yaw, _, _) = transform.rotation.to_euler(YXZ);
        let yaw_rot = Quat::from_rotation_y(yaw);
        let mut horizontal = yaw_rot * Vec3::new(delta.x, 0.0, delta.z);
        if horizontal.length_squared() > 0.0 { horizontal = horizontal.normalize(); }

        transform.translation += horizontal * speed * delta_time + Vec3::Y * delta.y * speed * delta_time;
    }
}