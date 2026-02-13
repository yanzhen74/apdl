//! APDL Link Simulation Kernel
//!
//! This crate provides the simulation kernel for protocol links in the APDL system.

pub mod channel;
pub mod data_generator;
pub mod demultiplex;
pub mod frame_disassembler;
pub mod layered_disassembler;
pub mod receiver;
pub mod simulator;
pub mod traffic_generator;

pub use channel::Channel;
pub use data_generator::{
    BoundaryValueStrategy, ConstraintHandler, ConstraintValidator, DataGenerator, DataImporter,
    FixedStrategy, GenerationStrategy, RandomStrategy, SequentialStrategy,
};
pub use demultiplex::{
    ChannelState, Demultiplexer, ReorderBuffer, SequenceValidator, ValidationResult,
};
pub use frame_disassembler::{extract_bit_field, FieldValidator, FrameDisassembler};
pub use layered_disassembler::{DisassembleResult, LayerData, LayeredDisassembler, ValidationError};
pub use receiver::{FrameSynchronizer, ReceiveBuffer, SyncMode};
pub use simulator::ProtocolSimulator;
pub use traffic_generator::TrafficGenerator;
