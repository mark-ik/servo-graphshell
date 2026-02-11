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

    /// Update physics configuration
    UpdateConfig(PhysicsConfig),
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

/// Process a single command, mutating engine/graph state and returning an optional response.
fn process_command(
    engine: &mut PhysicsEngine,
    graph: &mut Option<Graph>,
    command: PhysicsCommand,
) -> Option<PhysicsResponse> {
    match command {
        PhysicsCommand::UpdateGraph(new_graph) => {
            *graph = Some(new_graph);
            None
        }
        PhysicsCommand::Step(_dt) => {
            // Step is handled by the worker loop each frame
            None
        }
        PhysicsCommand::Toggle => {
            engine.toggle();
            Some(PhysicsResponse::IsRunning(engine.is_running))
        }
        PhysicsCommand::Pause => {
            engine.pause();
            Some(PhysicsResponse::IsRunning(false))
        }
        PhysicsCommand::Resume => {
            engine.resume();
            Some(PhysicsResponse::IsRunning(true))
        }
        PhysicsCommand::UpdateConfig(new_config) => {
            engine.config = new_config;
            None
        }
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
            if let Some(response) = process_command(&mut engine, &mut graph, command) {
                let _ = response_tx.send(response);
            }
        }

        // Run physics step if we have a graph
        if let Some(ref mut g) = graph {
            if engine.is_running {
                engine.step(g, target_dt);

                // Collect updated positions
                let positions: HashMap<NodeKey, Point2D<f32>> = g
                    .nodes()
                    .map(|(key, node)| (key, node.position))
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

#[cfg(test)]
mod tests {
    use super::*;
    use euclid::default::Point2D;

    fn make_engine() -> PhysicsEngine {
        PhysicsEngine::new(PhysicsConfig::default(), 1000.0)
    }

    #[test]
    fn test_toggle_returns_running_state() {
        let mut engine = make_engine();
        let mut graph = None;

        // Engine starts running, toggle should pause it
        let resp = process_command(&mut engine, &mut graph, PhysicsCommand::Toggle);
        assert!(matches!(resp, Some(PhysicsResponse::IsRunning(false))));
        assert!(!engine.is_running);

        // Toggle again should resume
        let resp = process_command(&mut engine, &mut graph, PhysicsCommand::Toggle);
        assert!(matches!(resp, Some(PhysicsResponse::IsRunning(true))));
        assert!(engine.is_running);
    }

    #[test]
    fn test_pause_and_resume() {
        let mut engine = make_engine();
        let mut graph = None;

        let resp = process_command(&mut engine, &mut graph, PhysicsCommand::Pause);
        assert!(matches!(resp, Some(PhysicsResponse::IsRunning(false))));
        assert!(!engine.is_running);

        let resp = process_command(&mut engine, &mut graph, PhysicsCommand::Resume);
        assert!(matches!(resp, Some(PhysicsResponse::IsRunning(true))));
        assert!(engine.is_running);
    }

    #[test]
    fn test_update_graph_sets_graph() {
        let mut engine = make_engine();
        let mut graph = None;

        let mut g = Graph::new();
        g.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));

        let resp = process_command(&mut engine, &mut graph, PhysicsCommand::UpdateGraph(g));
        assert!(resp.is_none());
        assert!(graph.is_some());
        assert_eq!(graph.as_ref().unwrap().node_count(), 1);
    }

    #[test]
    fn test_update_config() {
        let mut engine = make_engine();
        let mut graph = None;

        let mut new_config = PhysicsConfig::default();
        new_config.repulsion_strength = 9999.0;

        let resp = process_command(
            &mut engine,
            &mut graph,
            PhysicsCommand::UpdateConfig(new_config),
        );
        assert!(resp.is_none());
        assert_eq!(engine.config.repulsion_strength, 9999.0);
    }

    #[test]
    fn test_step_command_is_noop() {
        let mut engine = make_engine();
        let mut graph = None;

        let resp = process_command(&mut engine, &mut graph, PhysicsCommand::Step(0.016));
        assert!(resp.is_none());
    }

    #[test]
    fn test_command_sequence() {
        let mut engine = make_engine();
        let mut graph = None;

        // Simulate a realistic command sequence
        let mut g = Graph::new();
        g.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        g.add_node("https://b.com".to_string(), Point2D::new(100.0, 100.0));
        process_command(&mut engine, &mut graph, PhysicsCommand::UpdateGraph(g));

        // Pause, update config, resume
        process_command(&mut engine, &mut graph, PhysicsCommand::Pause);
        assert!(!engine.is_running);

        let mut config = PhysicsConfig::default();
        config.damping = 0.5;
        process_command(&mut engine, &mut graph, PhysicsCommand::UpdateConfig(config));
        assert_eq!(engine.config.damping, 0.5);

        process_command(&mut engine, &mut graph, PhysicsCommand::Resume);
        assert!(engine.is_running);
        assert_eq!(graph.as_ref().unwrap().node_count(), 2);
    }
}
