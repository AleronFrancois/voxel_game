use bevy::prelude::*;



/// Adds a single directional light to the scene.
pub fn setup_lighting(mut commands: Commands) {
    commands.spawn((DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_rotation(
            Quat::from_euler(EulerRot::XYZ, -std::f32::consts::FRAC_PI_4, std::f32::consts::FRAC_PI_4, 0.0)
        ),
        GlobalTransform::default(),
    ));
}