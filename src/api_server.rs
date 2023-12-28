//! Representation of the k8s API server

use dslab_core::cast;
use crate::pod::Pod;
use dslab_core::context::SimulationContext;
use dslab_core::event::Event;
use dslab_core::handler::EventHandler;

pub struct APIServer {

}

impl APIServer {
    pub fn new() {

    }

    // Get next pod in the PodQueue
    pub fn get_pod(&self) -> Pod {

    }

    // Get list of working nodes
    pub fn get_nodes(&self) {

    }
}

impl EventHandler for APIServer {
    fn on(&mut self, event: Event) {
        // processing of APIServer events
    }
}