use bevy::window::CursorGrabMode;
use bevy::window::CursorOptions;
use bevy::window::PrimaryWindow;
use bevy::window::WindowFocused;
use bevy::prelude::*;


// Grabs cursor event
#[derive(Event, Deref)]
pub struct GrabEvent(bool);


/*
    Grabs cursor and sets visibility.
    Param..: grab, window
    Return.: none
*/
pub fn apply_grab(
    grab: On<GrabEvent>,
    mut query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if let Ok(mut cursor_options) = query.single_mut() {
        if **grab {
            cursor_options.visible = false;
            cursor_options.grab_mode = CursorGrabMode::Locked;
        } 
        else {
            cursor_options.visible = true;
            cursor_options.grab_mode = CursorGrabMode::None;
        }
    }
}


/*
    Listens for window focus event and triggers the grab event.
    Param..: events, commands
    Return.: none
*/
pub fn focus_events(
    mut events: MessageReader<WindowFocused>,
    mut commands: Commands,
) {
    if let Some(event) = events.read().last() {
        commands.trigger(GrabEvent(event.focused));
    }
}


/*
    Toggles cursor grab and visibility manually when the user exists focused mode.
    Param..: window, commands
    Return.: none
*/
pub fn toggle_grab(
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    window.focused = !window.focused;
    commands.trigger(GrabEvent(window.focused));
}