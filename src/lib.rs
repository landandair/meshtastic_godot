use godot::prelude::*;
mod mt_node;
mod router;
mod godot_binds;
mod ipc;
mod packet_handler;
mod util;
mod connection;
mod api;

struct GodotMeshtastic;

#[gdextension]
unsafe impl ExtensionLibrary for GodotMeshtastic {}