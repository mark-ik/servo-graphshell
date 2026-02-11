Summary
I've implemented a complete physics configuration UI that allows real-time tuning of physics parameters:

What was added:

physics/worker.rs:36 - UpdateConfig command

Added new command variant to PhysicsCommand enum
Handles config updates in worker thread
app.rs:74-76 - UI state tracking

Added show_physics_panel: bool field to GraphBrowserApp
Initialized to false (panel hidden by default)
app.rs:352-365 - Config management methods

update_physics_config() - Sends new config to worker and updates local state
toggle_physics_panel() - Shows/hides the UI panel
input/mod.rs:17-54 - P key binding

Added P key to toggle physics config panel
Works in both graph and detail views
render/mod.rs:147-250 - Physics config panel UI

Configurable parameters with sliders:
Repulsion Strength (0-20000, logarithmic scale)
Spring Strength (0-1.0)
Spring Rest Length (10-500px)
Velocity Damping (0-1.0)
Velocity Threshold (0.0001-0.1, logarithmic)
Auto-pause Delay (0-30s)
Real-time config updates on slider changes
"Reset to Defaults" button
Running/Paused status indicator
desktop/gui.rs:674 - Panel integration

Renders physics panel in both graph and detail views
Placed after main view rendering
render/mod.rs:136 - Updated controls hint

Added "P: Physics Settings" to on-screen controls
How to use:

Press P to open/close the physics configuration panel
Adjust sliders to see real-time effects on the graph layout
Click "Reset to Defaults" to restore original values
Changes are instantly applied to the physics simulation
All 49 tests pass and the binary compiles successfully!