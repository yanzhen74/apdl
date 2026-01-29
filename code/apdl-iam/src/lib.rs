//! APDL Interaction Access Module
//!
//! This crate provides user interaction and access interfaces for the APDL system.

pub mod api;
pub mod cli;
pub mod gui;

pub use api::RestApiServer;
pub use cli::CommandLineInterface;
pub use gui::GuiApp;
