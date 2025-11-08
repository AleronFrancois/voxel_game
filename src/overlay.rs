use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::player::Player;


#[derive(Resource, Default)]
pub struct TextOverlay(pub Option<Entity>);


pub struct StatsOverlayPlugin;


impl Plugin for StatsOverlayPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TextOverlay>()
            .add_systems(Update, stats_overlay);
    }
}


fn stats_overlay(
    mut commands: Commands,
    player_query: Query<&GlobalTransform, With<Player>>,
    mut text_query: Query<(&mut Text2d, &mut Transform)>,
    mut text_overlay: ResMut<TextOverlay>,
    window_query: Query<&Window>,
) {
    let player_transform = if let Ok(transform) = player_query.single() { transform }
    else { return; };

    let position = player_transform.translation();
    let player_position = format!("XYZ: {:.1} {:.1} {:.1}", position.x, position.y, position.z);
    let window = window_query.single().expect("No primary window found");

    let text_position = Vec3::new(
        -window.width() / 2.0 + 1.0, 
        window.height() / 2.0 - 20.0, 
        0.0,
    );

    if let Some(entity) = text_overlay.0 {
        if let Ok((mut text, mut transform)) = text_query.get_mut(entity) {
            text.0 = player_position.clone();
            transform.translation = text_position;
        }
        return;
    }

    let entity = commands.spawn((
        Text2d(player_position.clone()),
        TextColor(Color::WHITE),
        TextFont {
            font_size: 16.0,  // original font size
            ..Default::default()
        },
        Transform::from_translation(text_position),
        Anchor::TOP_LEFT,
    )).id();

    text_overlay.0 = Some(entity);
}
