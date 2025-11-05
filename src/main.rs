use bevy::pbr::wireframe::WireframePlugin;
use bevy::render::RenderDebugFlags;
use bevy::window::PresentMode;
use bevy::input::common_conditions::input_just_released;
use bevy::prelude::*;
use crate::fps_counter::FpsCounterPlugin;

mod fps_counter;
mod crosshair;
mod camera;
mod player;
mod chunk;
mod world;
mod block;
mod light;
mod settings;


fn main() {
    let mut game = App::new(); 

    // Plugins
    game.add_plugins(DefaultPlugins.set(WindowPlugin { 
        primary_window: Some(Window {
            title: "Voxel Game".to_string(),
            present_mode: PresentMode::Immediate, 
            ..default()
        }),
        ..default()
    }));
    game.add_plugins(FpsCounterPlugin);
    game.add_plugins(WireframePlugin {
        debug_flags: RenderDebugFlags::empty(),
    });

    // Observers
    game.add_observer(settings::apply_grab);

    // Resources
    game.insert_resource(world::WorldChunks::default());
    game.insert_resource(world::ChunkEntities::default());
    game.insert_resource(world::ChunkQueue::default());
    game.insert_resource(world::PlayerChunk::default());
    game.insert_resource(player::JumpState::new());
    game.insert_resource(player::PlayerSettings::default());

    // Load systems on startup
    //game.add_systems(Startup, camera::spawn_camera);
    game.add_systems(Startup, light::setup_lighting);
    game.add_systems(Startup, crosshair::setup_crosshair);

    game.add_systems(Startup, player::spawn_player); // Spawn player and camera attached

    // Load systems on update frame
    game.add_systems(Update, settings::focus_events);
    game.add_systems(Update, settings::toggle_grab.run_if(input_just_released(KeyCode::Escape)));

    //game.add_systems(Update, camera::camera_look);
    //game.add_systems(Update, camera::camera_movement.after(camera::camera_look));

    game.add_systems(Update, player::camera_look); // Use players camera look function


    game.add_systems(Update, world::queue_chunks);
    game.add_systems(Update, world::load_chunks.after(world::queue_chunks));
    game.add_systems(Update, world::unload_chunks.after(world::load_chunks));
    game.add_systems(Update, player::player_movement);
    
    game.run();
}