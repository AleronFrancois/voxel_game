use bevy::pbr::wireframe::WireframePlugin;
use bevy::render::RenderDebugFlags;
use bevy::window::PresentMode;
use bevy::input::common_conditions::input_just_released;
use bevy::prelude::*;
use crate::fps_counter::FpsCounterPlugin;
use avian3d::prelude::*;

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

    // --- Plugins
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
    game.add_plugins(PhysicsPlugins::default());

    // --- Observers
    game.add_observer(settings::apply_grab);

    // --- Resources
    game.insert_resource(world::WorldChunks::default());
    game.insert_resource(world::ChunkEntities::default());
    game.insert_resource(world::ChunkQueue::default());
    game.insert_resource(world::PlayerChunk { last_chunk: glam::IVec3::new(i32::MIN, 0, i32::MIN) });
    game.insert_resource(player::PlayerSettings::default());
    game.insert_resource(player::JumpState::default());

    // --- Load systems on startup
    game.add_systems(Startup, light::setup_lighting);
    game.add_systems(Startup, crosshair::setup_crosshair);
    game.add_systems(Startup, player::spawn_player); 

    // --- Load systems on update frame
    game.add_systems(Update, settings::focus_events);
    game.add_systems(Update, settings::toggle_grab.run_if(input_just_released(KeyCode::Escape)));
    game.add_systems(Update, player::camera_look);
    game.add_systems(Update, player::player_movement);
    // Queue, load and unload chunks around the players position
    game.add_systems(Update, world::queue_chunks.after(player::player_movement));
    game.add_systems(Update, world::load_chunks.after(world::queue_chunks));
    game.add_systems(Update, world::unload_chunks.after(world::load_chunks));
    
    game.run();
}