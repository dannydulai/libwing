mod console;
mod node;
mod ffi;

pub use console::{WingConsole, DiscoveryInfo};
pub use node::{NodeDefinition, NodeData, NodeType, NodeUnit};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid data received")]
    InvalidData,
    #[error("Connection error")]
    ConnectionError,
}

pub type Result<T> = std::result::Result<T, Error>;
