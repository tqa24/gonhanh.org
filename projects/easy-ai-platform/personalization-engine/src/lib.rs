//! Personalization Engine Library
//!
//! Core modules:
//! - `api`: REST API endpoints for personalization and admin
//! - `connectors`: Database and external service connectors
//! - `engine`: Lead scoring, content rules, and orchestration
//! - `models`: Data structures and DTOs

pub mod api;
pub mod config;
pub mod connectors;
pub mod engine;
pub mod models;

pub use config::Config;
