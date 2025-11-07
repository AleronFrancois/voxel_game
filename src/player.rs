use std::f32::consts::FRAC_PI_2;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::window::PrimaryWindow;
use avian3d::prelude::*;
use bevy::prelude::*;
use EulerRot::YXZ;


#[derive(Component)]
pub struct Player;


#[derive(Resource)]
pub struct PlayerSettings {
    pub move_speed: f32,
}


#[derive(Component)]
pub struct CameraSettings {
    pub sensitivity: f32,
}


#[derive(Resource, Default)]
pub struct JumpState {
    pub should_jump: bool,
    pub jump_strength: f32,
}


impl JumpState {
    pub fn default() -> Self {
        Self {
            should_jump: false,
            jump_strength: 35.0,
        }
    }
}


impl PlayerSettings {
    pub fn default() -> Self {
        Self {
            move_speed: 15.0,
        }
    }
}


impl CameraSettings {
    pub fn default() -> Self {
        Self {
            sensitivity: 0.4,
        }
    }
}


/// Spawns player along with its camera and mesh.
pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let start_position = Vec3::new(0.0, 80.0, 0.0);

    // Add mesh data to asset storage
    let mesh_handle = meshes.add(Sphere::default().mesh().ico(5).unwrap());
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        perceptual_roughness: 0.5,
        ..Default::default()
    });

    // Spawn player entity
    let player_entity = commands.spawn((
        Player,
        Transform::from_translation(start_position),
        GlobalTransform::default(),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        LinearVelocity::default(),
        Restitution::new(0.0),
        LockedAxes::ROTATION_LOCKED,
        Friction::ZERO,
    )).id();

    // Spawn camera entity
    let camera_entity = commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        CameraSettings::default(),
        GlobalTransform::default(),
    )).id();

    // Spawn mesh entity
    let mesh_entity = commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 2.0, 1.0),
            ..Default::default()
        },
        GlobalTransform::default(),
    )).id();

    // set players children
    commands.entity(player_entity).add_child(mesh_entity);
    commands.entity(player_entity).add_child(camera_entity);
}


/// Handles player movement.
pub fn player_movement(
    mut query: Query<&mut LinearVelocity, With<Player>>,
    camera_query: Query<&Transform, With<Camera3d>>,
    input: Res<ButtonInput<KeyCode>>,
    player_settings: Res<PlayerSettings>,
    mut jump_state: ResMut<JumpState>,
    time: Res<Time>,
) {
    let deltatime = time.delta_secs();
    let camera_transform = if let Ok(camera) = camera_query.single() {
        camera
    } 
    else {
        return;
    };

    // Calculate forward & right from camera yaw
    let yaw = camera_transform.rotation.to_euler(EulerRot::YXZ).0;
    let forward = Vec3::new(yaw.sin(), 0.0, yaw.cos()).normalize();
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin()).normalize();

    // Read keyboard input
    let mut direction = Vec3::ZERO;
    let mut speed = player_settings.move_speed;
    if input.pressed(KeyCode::KeyW) { direction.z -= 1.0; }
    if input.pressed(KeyCode::KeyS) { direction.z += 1.0; }
    if input.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
    if input.pressed(KeyCode::KeyD) { direction.x += 1.0; }
    if input.pressed(KeyCode::ShiftLeft) { speed *= 2.0; }
    if input.pressed(KeyCode::Space) { 
        jump_state.should_jump = true;
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
    }

    let horizontal_velocity = (forward * direction.z + right * direction.x) * speed;

    for mut linear_velocity in query.iter_mut() {
        let grounded = linear_velocity.y.abs() < 0.05;

        // Apply horizontal movement
        linear_velocity.x = horizontal_velocity.x;
        linear_velocity.z = horizontal_velocity.z;

        linear_velocity.y += -9.8 * 5.0 * deltatime;

        // Clamp upward velocity
        if linear_velocity.y > 0.0 {
            linear_velocity.y = 0.0;
        }

        // Jump
        if grounded && jump_state.should_jump {
            linear_velocity.y = jump_state.jump_strength;
            jump_state.should_jump = false;
        }
    }
}


/// Handles mouse camera look
pub fn camera_look(
    camera: Single<(&mut Transform, &CameraSettings)>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if !window.focused { return; }

    let (mut transform, camera_data) = camera.into_inner();
    let sensitivity =
        camera_data.sensitivity * (window.width().min(window.height()) / window.height());

    let deltatime = time.delta_secs();
    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(YXZ);

    pitch -= mouse_motion.delta.y * deltatime * sensitivity;
    pitch = pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
    yaw -= mouse_motion.delta.x * deltatime * sensitivity;

    transform.rotation = Quat::from_euler(YXZ, yaw, pitch, 0.0);
}