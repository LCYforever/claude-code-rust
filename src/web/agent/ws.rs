//! WebSocket Skeleton
//!
//! Provides WebSocket handler and ConnectionManager skeleton
//! for future real-time push scenarios.

use dashmap::DashMap;
use tokio::sync::mpsc;

/// WebSocket message type
#[derive(Debug, Clone)]
pub enum WsMessage {
    Text(String),
    Close,
}

/// Connection manager for WebSocket connections
pub struct ConnectionManager {
    connections: DashMap<String, mpsc::UnboundedSender<WsMessage>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }

    /// Register a new connection
    pub fn add_connection(&self, id: String, tx: mpsc::UnboundedSender<WsMessage>) {
        self.connections.insert(id, tx);
    }

    /// Remove a connection
    pub fn remove_connection(&self, id: &str) {
        self.connections.remove(id);
    }

    /// Send message to a specific connection
    pub fn send_to(&self, id: &str, msg: WsMessage) -> bool {
        if let Some(tx) = self.connections.get(id) {
            tx.send(msg).is_ok()
        } else {
            false
        }
    }

    /// Broadcast message to all connections
    pub fn broadcast(&self, msg: WsMessage) {
        for entry in self.connections.iter() {
            let _ = entry.value().send(msg.clone());
        }
    }

    /// Get number of active connections
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
