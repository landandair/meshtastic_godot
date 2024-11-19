use godot::prelude::*;
mod godot_binds;
mod api;
mod mesh_connection;

struct GodotMeshtastic;

#[gdextension]
unsafe impl ExtensionLibrary for GodotMeshtastic {}