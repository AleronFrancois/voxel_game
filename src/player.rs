use bevy::{input::mouse::AccumulatedMouseMotion, math::sampling::shape_sampling, prelude::*, window::PrimaryWindow};
use EulerRot::YXZ;


#[derive(Component)]
pub struct Player;


#[derive(Resource, Default)]
pub struct JumpState {
    should_jump: bool,
}


#[derive(Resource)]
pub struct PlayerSettings {
    pub move_speed: f32,
    pub jump_strength: f32,
}


impl JumpState {
    pub fn new() -> Self {
        Self {
            should_jump: false,
        }
    }
}


impl PlayerSettings {
    pub fn default() -> Self {
        Self {
            move_speed: 50.0,
            jump_strength: 10.0,
        }
    }
}


#[derive(Component)]
pub struct CameraSettings {
    pub speed: f32,
    pub sensitivity: f32,
}


pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let start_position = Vec3::new(0.0, 18.0, -1.0);
    let mesh_handle = meshes.add(Sphere::default().mesh().ico(5).unwrap());
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        perceptual_roughness: 0.5,
        ..Default::default()
    });

    let camera_entity = commands.spawn((
        Camera3d::default(),
        Transform::from_translation(start_position),
        GlobalTransform::default(),
        CameraSettings { speed: 50.0, sensitivity: 0.4 },
    )).id();

    let mesh_entity = commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform {
            translation: Vec3::new(0.0, -1.0, 0.0),
            ..Default::default()
        },
        GlobalTransform::default(),
    )).id();

    let player_entity = commands.spawn((
        Player,
        Transform::from_translation(start_position),
        GlobalTransform::default(),
    )).id();

    commands.entity(player_entity).add_child(camera_entity);
    commands.entity(camera_entity).add_child(mesh_entity);
}


pub fn player_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut player: Query<&mut Transform, With<Player>>,
    settings: Res<PlayerSettings>,
    jump_state: ResMut<JumpState>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = player.single_mut() {
        let mut direction = Vec3::ZERO;

        if input.pressed(KeyCode::KeyW) { direction.z -= 1.0; }
        if input.pressed(KeyCode::KeyS) { direction.z += 1.0; }
        if input.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        if input.pressed(KeyCode::KeyD) { direction.x += 1.0; }
        if input.pressed(KeyCode::KeyQ) { direction.y -= 1.0; }
        if input.pressed(KeyCode::KeyE) { direction.y += 1.0; }
        if input.pressed(KeyCode::Space) && !jump_state.should_jump {
            transform.translation.y += settings.jump_strength;
        }

        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
        }

        transform.translation += direction * settings.move_speed * time.delta_secs();
    }
}


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