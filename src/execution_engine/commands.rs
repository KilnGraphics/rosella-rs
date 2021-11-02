//! Commands are the last intermediate representation before conversion into vulkan command buffers.
//! The IR is designed to be a direct mapping to vulkan commands with only placeholders for
//! specializable resources and external synchronization for them left unresolved.

