# Wing Rust Library

This is the Rust implementation of the Wing library for discovering and controlling Behringer Wing mixers over the network.

## Features

- Full implementation of the Wing Native protocol
- Safe Rust wrapper around all functionality
- FFI layer providing C API compatibility
- Discovery protocol implementation
- Async-friendly network communication
- Type-safe node definitions and data handling

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
wing = { path = "../rust" }  # Adjust path as needed
```

### Basic Example

```rust
use wing::{WingConsole, NodeDefinition, NodeData};

fn main() -> wing::Result<()> {
    // Discover Wing devices
    let devices = WingConsole::scan(true);
    if devices.is_empty() {
        println!("No Wing devices found!");
        return Ok(());
    }

    // Connect to first device found
    let mut console = WingConsole::connect(&devices[0].ip)?;
    
    // Set up callbacks
    console.on_node_definition = Some(Box::new(|def: NodeDefinition| {
        println!("Node Definition: {} ({})", def.name, def.id);
    }));
    
    console.on_node_data = Some(Box::new(|id: u32, data: NodeData| {
        println!("Node {} data updated", id);
    }));

    // Request some node data
    console.request_node_definition(0)?;
    
    // Main event loop
    loop {
        console.read()?;
    }
}
```

## Building

The library uses standard Rust tooling:

```bash
cd rust
cargo build
cargo test
```

## FFI/C API

The library provides a complete C API through FFI bindings. This allows seamless integration with existing C/C++ code while maintaining memory safety through Rust's ownership model.

See wing_c_api.h for the complete C API documentation.
