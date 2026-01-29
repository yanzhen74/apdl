//! APDL Specification Auto-generation Kit
//!
//! This crate provides automatic generation of protocol specifications for the APDL system.

pub mod exporters;
pub mod generator;
pub mod templates;

pub use exporters::MarkdownExporter;
pub use generator::SpecGenerator;
pub use templates::TemplateEngine;
