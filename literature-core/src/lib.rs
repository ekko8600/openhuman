//! Agent-first personal literature knowledge base core.
//!
//! This crate extracts the OpenHuman core idea—small domain operations exposed
//! through deterministic command/tool boundaries—and narrows it to local
//! literature-library construction for coding agents such as Codex, Cursor, and
//! opencode.

pub mod cli;
pub mod documents;
pub mod index;
pub mod manifest;
pub mod parser;
pub mod rpc;
pub mod store;
pub mod types;

pub use documents::{add_document, get_document, list_documents, remove_document};
pub use index::{build_index, search_index};
pub use store::LiteratureStore;
pub use types::*;
