//! [![github]](https://github.com/dannydulai/libwing)&ensp;[![crates-io]](https://crates.io/crates/libwing)&ensp;[![docs-rs]](https://docs.rs/libwing)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

mod console;
mod node;
mod ffi;
mod propmap;

pub use console::{WingConsole, DiscoveryInfo};
pub use node::{WingNodeDef, WingNodeData, NodeType, NodeUnit};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid data received")]
    InvalidData,
    #[error("Connection error")]
    ConnectionError,
}

pub enum WingResponse {
    RequestEnd,
    NodeDef(WingNodeDef),
    NodeData(i8, i32, WingNodeData),
}
