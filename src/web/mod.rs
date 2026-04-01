//! Web Module - Plugin Marketplace Web Interface & Agent API
//!
//! This module provides a web server for the plugin marketplace
//! and the Agent intelligent assistant API using Axum framework.

pub mod server;
pub mod routes;
pub mod handlers;
pub mod models;
pub mod templates;
pub mod agent;

pub use server::WebServer;
pub use models::*;
pub use agent::AgentWebState;
