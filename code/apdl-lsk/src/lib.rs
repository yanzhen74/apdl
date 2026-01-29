//! APDL Link Simulation Kernel
//!
//! This crate provides the simulation kernel for protocol links in the APDL system.

pub mod channel;
pub mod simulator;
pub mod traffic_generator;

pub use channel::Channel;
pub use simulator::ProtocolSimulator;
pub use traffic_generator::TrafficGenerator;
