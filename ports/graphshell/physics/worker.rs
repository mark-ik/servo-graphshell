/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Physics worker thread for non-blocking simulation.

use crate::graph::{Graph, NodeKey};
use crate::physics::{PhysicsConfig, PhysicsEngine};
use crossbeam_channel::{Receiver, Sender};
use euclid::default::Point2D;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

/// Command to send to the physics worker
pub enum PhysicsCommand {
    /// Update graph state (full sync, only when structure changes)
    UpdateGraph(Graph),
    
    /// Step physics simulation (lightweight, no clone)
    Step(f32),
    
    /// Toggle physics on/off
    Toggle,
    
    /// Pause physics
    Pause,
    
    /// Resume physics
    Resume,
    
    /// Update viewport size
    UpdateViewport(f32),
    
    /// Shutdown the worker
    Shutdown,
}

/// Response from the physics worker
pub enum PhysicsResponse {
    /// Updated node positions
    NodePositions(HashMap<NodeKey, Point2D<f32>>),
    
    /// Physics is running status
    IsRunning(bool),
}

/// Physics worker that runs on a background thread
pub struct PhysicsWorker {
    command_tx: Sender<PhysicsCommand>,
    response_rx: Receiver<PhysicsResponse>,
}

impl PhysicsWorker {
    /// Create and start a new physics worker
    pub fn new(config: PhysicsConfig, viewport_diagonal: f32) -> Self {
        let (command_tx, command_rx) = crossbeam_channel::unbounded();
        let (response_tx, response_rx) = crossbeam_channel::unbounded();
        
        // Spawn the worker thread
        thread::spawn(move || {
            run_physics_worker(config, viewport_diagonal, command_rx, response_tx);
        });
        
        Self {
            command_tx,
            response_rx,
        }
    }
    
    /// Send a command to the physics worker
    pub fn send_command(&self, command: PhysicsCommand) {
        let _ = self.command_tx.send(command);
    }
    
    /// Try to receive a response (non-blocking)
    pub fn try_recv_response(&self) -> Option<PhysicsResponse> {
        self.response_rx.try_recv().ok()
    }
}

/// Run the physics simulation in a background thread
fn run_physics_worker(
    config: PhysicsConfig,
    viewport_diagonal: f32,
    command_rx: Receiver<PhysicsCommand>,
    response_tx: Sender<PhysicsResponse>,
) {
    let mut engine = PhysicsEngine::new(config, viewport_diagonal);
    let mut graph: Option<Graph> = None;
    let target_dt = 1.0 / 60.0; // 60 FPS target
    
    loop {
        let start = std::time::Instant::now();
        
        // Process all pending commands
        while let Ok(command) = command_rx.try_recv() {
            match command {
                PhysicsCommand::UpdateGraph(new_graph) => {
                    graph = Some(new_graph);
                }
                PhysicsCommand::Step(_dt) => {
                    // Step command is handled in the main loop below
                    // (this is just for compatibility, we step every frame)
                }
                PhysicsCommand::Toggle => {
                    engine.toggle();
                    let _ = response_tx.send(PhysicsResponse::IsRunning(engine.is_running));
                }
                PhysicsCommand::Pause => {
                    engine.pause();
                    let _ = response_tx.send(PhysicsResponse::IsRunning(false));
                }
                PhysicsCommand::Resume => {
                    engine.resume();
                    let _ = response_tx.send(PhysicsResponse::IsRunning(true));
                }
                PhysicsCommand::UpdateViewport(diagonal) => {
                    engine.update_viewport(diagonal);
                }
                PhysicsCommand::Shutdown => {
                    return;
                }
            }
        }
        
        // Run physics step if we have a graph
        if let Some(ref mut g) = graph {
            if engine.is_running {
                engine.step(g, target_dt);
                
                // Collect updated positions
                let positions: HashMap<NodeKey, Point2D<f32>> = g
                    .nodes()
                    .map(|node| (node.id, node.position))
                    .collect();
                
                // Send positions back to main thread
                let _ = response_tx.send(PhysicsResponse::NodePositions(positions));
            }
        }
        
        // Sleep to maintain target framerate
        let elapsed = start.elapsed();
        if elapsed < Duration::from_secs_f32(target_dt) {
            thread::sleep(Duration::from_secs_f32(target_dt) - elapsed);
        }
    }
}
