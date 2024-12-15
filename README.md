# Meshtastic Plugin for Godot
How to use the plugin
------------------------
1. Build the plugin
- Install Rust https://www.rust-lang.org/tools/install
- Navigate to addon folder- addons/meshtastic_godot/release
- Run the command "cargo build --release"
- Ensure the library file in the target folder is a path in the gdextension file
2. Run the example
- Open up Godot and load the project file in this folder
- Run or export the project in godot

**Example project overview**

The example project contains the following:
- Basic text client example for demonstrating sending and receiving text over meshtastic
- tic tac toe example for demonstrating binary transfer over meshtastic
- Also contains general usage examples of the plugin including how to connect to a radio and how to send data to it and disconnect from it