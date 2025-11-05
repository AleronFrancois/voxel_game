use bevy::prelude::*;


/// Displays crosshair.
pub fn setup_crosshair(
    mut commands: Commands,
) { 
    let size: f32 = 10.0;     // Size of crosshair
    let thickness: f32 = 1.0; // Thickness of crosshair

    // Spawn 2D camera to view crosshair
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..Default::default()
        },
        Transform::default(),
        GlobalTransform::default(),
    ));

    // Horizontal line
    commands.spawn(Sprite {
        custom_size: Some(Vec2::new(size, thickness)),
        color: Color::WHITE,
        ..Default::default()
    });

    // Vertical line
    commands.spawn(Sprite {
        custom_size: Some(Vec2::new(thickness, size)),
        color: Color::WHITE,
        ..Default::default()
    });
}