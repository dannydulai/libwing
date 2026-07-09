# libwing Specification

A Rust library for discovering and controlling Behringer Wing digital mixing consoles over a local network.

## Overview

libwing implements the Wing Native Binary Protocol, providing:

- **Network discovery** of Wing devices via UDP broadcast
- **Persistent TCP connections** for real-time control
- **Property read/write** for all mixer parameters (faders, mutes, EQ, routing, etc.)
- **Real-time monitoring** of property changes
- **Meter data** (level indicators) via subscription
- **Property schema** with 78,000+ name-to-ID mappings
- **C FFI bindings** for use from C/C++/Dart/Flutter

## Supported Hardware

| Model | Identifier |
|-------|------------|
| Behringer Wing | `wing` |
| Wing Compact | `wing-compact` |
| Wing Rack | `wing-rack` |
| Wing BK | `wing-bk` |
| NGC Full | `ngc-full` |

Tested against firmware v3.0.5.

## Protocol

### Transport

- **Discovery**: UDP broadcast on port 2222
- **Control**: TCP on port 2222
- **Keep-alive**: 7-second heartbeat (data), 3-second heartbeat (meters)

### Binary Encoding

- Property IDs are 32-bit integers encoded with escape sequences (0xdf prefix, 0xde escape byte)
- Command codes span 0xd0–0xdf for data operations, 0xda–0xdd for control
- Values use variable-length encoding: 0x00–0x3f for small ints, 0x80–0xbf for strings, 0xd3–0xd6 for typed data

## Core API

### Discovery

```rust
// Broadcast scan — returns list of DiscoveryInfo (ip, name, model, serial, firmware)
let devices = WingConsole::scan(stop_on_first: bool) -> Vec<DiscoveryInfo>;
```

### Connection

```rust
// Connect to a specific IP, or auto-discover if None
let wing = WingConsole::connect(host: Option<&str>) -> Result<WingConsole>;
```

### Reading

```rust
// Blocking read — returns property changes, schema definitions, or end markers
wing.read() -> Result<WingResponse>

enum WingResponse {
    RequestEnd,                      // End of multi-message response
    NodeDef(WingNodeDef),            // Property schema definition
    NodeData(i32, WingNodeData),     // Property value (id + value)
}
```

### Writing

```rust
wing.set_string(id: i32, value: &str) -> Result<()>
wing.set_float(id: i32, value: f32) -> Result<()>
wing.set_int(id: i32, value: i32) -> Result<()>
```

### Schema Queries

```rust
wing.request_node_definition(id: i32) -> Result<()>   // Request schema for a property
wing.request_node_data(id: i32) -> Result<()>          // Request current value
wing.name_to_id(name: &str) -> Option<i32>             // "/ch/1/mute" → numeric ID
wing.name_to_def(name: &str) -> Option<&WingNodeDef>   // name → full definition
wing.id_to_defs(id: i32) -> Option<Vec<&WingNodeDef>>  // ID → definitions
```

### Meters

```rust
// Subscribe to meter groups
wing.request_meter(meters: &[Meter]) -> Result<i32>

// Blocking read of meter levels
wing.read_meters() -> Result<(i32, Vec<f32>)>

enum Meter {
    Channel(u8), Aux(u8), Bus(u8), Main(u8), Matrix(u8),
    Dca(u8), Fx(u8), Source(u8), Output(u8), Monitor, Rta,
    Channel2(u8), Aux2(u8), Bus2(u8), Main2(u8), Matrix2(u8),
}
```

## Data Model

### WingNodeDef (Property Schema)

| Field | Type | Description |
|-------|------|-------------|
| id | i32 | Numeric property identifier |
| parent_id | i32 | Parent node ID |
| index | i32 | Index within parent |
| name | String | Short name |
| long_name | String | Human-readable name |
| node_type | NodeType | Value type (see below) |
| unit | NodeUnit | Display unit (dB, %, ms, Hz, etc.) |
| read_only | bool | Whether the property is writable |

### NodeType

- `Node` — container (no value, has children)
- `LinearFloat` / `LogarithmicFloat` / `FaderLevel` — float with min/max/steps
- `Integer` — integer with min/max/steps
- `StringEnum` / `FloatEnum` — enumerated values
- `String` — free-form text

### NodeUnit

None, Db, Percent, Milliseconds, Hertz, Meters, Seconds, Octaves

## Property Map

The library embeds a static HashMap of ~78,000 property mappings compiled from the Wing schema. This enables name-based lookups (e.g., `/ch/1/fader`) without querying the device. The map is lazily initialized on first access.

An `empty-propmap.rs` alternative exists for reduced binary size when name lookups are not needed.

## C FFI

A complete C API is exposed via `extern "C"` functions and documented in `libwing.h`. Key functions mirror the Rust API:

- `wing_discover_scan()` / `wing_discover_get_ip()` / `wing_discover_get_name()`
- `wing_console_connect()` / `wing_console_read()` / `wing_console_read_meters()`
- `wing_console_set_float()` / `wing_console_set_string()` / `wing_console_set_int()`
- `wing_console_name_to_id()` / `wing_console_name_to_def()`
- `wing_response_get_type()` / `wing_node_data_get_float()` / etc.

All returned pointers use opaque wrapper types. Memory is caller-managed via corresponding `_free()` functions.

## Build Targets

- **rlib** — Rust library for direct Rust consumption
- **cdylib** — shared library (`.dylib`/`.so`/`.dll`) for FFI consumers

## CLI Tools (Examples)

| Tool | Purpose |
|------|---------|
| `wingprop` | Get/set individual properties. Supports JSON output (`-j`). |
| `wingmon` | Real-time monitor — prints all property changes to stdout. |
| `wingschema` | Generates `propmap.rs` and `propmap.jsonl` from a live device. |
| `wingmeters` | GUI app displaying real-time channel meters (uses eframe). |

## Thread Safety

All `WingConsole` operations are thread-safe via `Arc<Mutex>`. Separate threads can handle data reads, meter reads, and writes concurrently.

## Dependencies

| Crate | Purpose |
|-------|---------|
| socket2 0.5 | TCP/UDP socket operations |
| lazy_static 1.4 | Lazy propmap initialization |
| jzon 0.12.5 | JSON serialization |
| thiserror 2.0.11 | Error type derives |
| eframe 0.26.0 | GUI (dev dependency, wingmeters only) |
