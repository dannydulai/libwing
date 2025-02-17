//! [![github]](https://github.com/dannydulai/libwing)&ensp;[![crates-io]](https://crates.io/crates/libwing)&ensp;[![docs-rs]](https://docs.rs/libwing)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! # Libwing SDK Documentation
//!
//! Libwing is a C++ library for interfacing with Behringer Wing digital mixing
//! consoles. It provides functionality for discovering Wing consoles on the
//! network, connecting to them, reading/writing console parameters, and receiving
//! any changes made on the mixer itself.
//!
//! There is a C wrapper for this library. It generally follows the C++ API. You
//! can find it in `wing_c_api.h`.
//!
//! ## Basic Concepts
//!
//! The Wing console exposes its functionality through a tree of nodes. Each node has:
//! - A unique numeric ID
//! - A hierarchical path name (like a filesystem path)
//! - A type (string, float, integer, enum, etc.)
//! - Optional min/max values and units
//! - Read/write or read-only access
//!
//! ## Getting Started
//!
//! ### Connecting
//! If you have a Wing's IP address, you can connect to it:
//!
//! ```rust
//! WingConsole wing = WingConsole::connect(Some("192.168.1.100"));
//! ```
//!
//! or just run with no IP address to discover the first Wing console on the network:
//!
//! ```rust
//! WingConsole wing = WingConsole::connect(None);
//! ```
//!
//! There is also `WingConsole::scan()` which can be used to scan for Wing mixers.
//!
//! ### Communication Model
//!
//! - `WingConsole::read()` will block and return you messages from the Wing mixer as they come in.
//!
//! - If the device is modified either physically or via another user of the API, the Wing device
//!   sends unsolicited `WingResponse::NodeData(channel, id, data)` messages. The `channel`
//!   parameter will always be the same unless you are using the metering API, which **libwing**
//!   does not implement yet.
//!
//! - You can request properties from the Wing device using `WingConsole.request_node_data()`,
//!   which will result in a `WingResponse::NodeData` being sent if your request was for a valid
//!   property. Note that you may get other properties as well, as the Wing device will send
//!   unsolicited property changes, so you may need to filter for your specific property change.
//!   After the NodeData is sent (or not), the Wing device will send a `WingResponse::RequestEnd`
//!   message.
//!
//! - You can request node definitions using `WingConsole::request_node_definition()`, which cause
//!   a `WingResponse::NodeDef` message to be read. **wingschema** uses this request to dump the
//!   schema. Again, unsolicited messages may be sent, so you may need to filter for your specific
//!   NodeDef. After the NodeDef is sent (or not), the Wing device will send a `WingResponse::RequestEnd`
//!
//! - You can set properties using the `WingConsole::set_*()` functions. These do not send any
//!   response back.
//!


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
    #[error("Failed to discover Wing console")]
    DiscoveryError,
}

pub enum WingResponse {
    RequestEnd,
    NodeDef(WingNodeDef),
    NodeData(i8, i32, WingNodeData),
}
