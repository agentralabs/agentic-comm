//! All MCP data types used by the server.

pub mod error;
pub mod message;
pub mod response;

// Re-export commonly used types for convenience.
pub use error::*;
pub use message::*;
pub use response::*;
