# libwing — Agent Guide

## What is this?

Rust library for discovering and controlling Behringer Wing digital mixing consoles over TCP/UDP. Published on crates.io as `libwing`. See SPEC.md for the full specification.

## Build & Run

```bash
cargo build                          # build library
cargo build --all-targets            # build library + all example tools
cargo build --example wingprop       # build a specific tool
cargo build --release                # optimized build
cargo test                           # run tests (requires no live hardware)
cargo doc --open                     # generate and view docs
```

The library produces both an `rlib` (Rust) and `cdylib` (C FFI shared library).

## Source Layout

```
src/
  lib.rs          — Public exports, Error enum, WingResponse enum
  console.rs      — WingConsole: connection, protocol encode/decode, read/write, meters, discovery
  node.rs         — WingNodeDef, WingNodeData, NodeType, NodeUnit — data model + serialization
  ffi.rs          — C FFI bindings (extern "C" functions, opaque handle types)
  propmap.rs      — Auto-generated: ~78k property name→def mappings (lazy_static HashMap)
  empty-propmap.rs — Stub propmap for smaller binaries when name lookups aren't needed
  propmap.jsonl   — JSON Lines reference of all property mappings

tools/
  wingprop.rs     — CLI: get/set individual properties
  wingmon.rs      — CLI: real-time property change monitor
  wingschema.rs   — CLI: regenerate propmap.rs + propmap.jsonl from a live device
  wingmeters.rs   — GUI: real-time channel meter display (eframe)
  utils.rs        — Shared helpers for CLI tools

libwing.h         — C API header
Discovery.md      — UDP discovery protocol notes
Wing-Remote-Protocols.pdf — Official Behringer protocol spec
```

## Property Map & Schema Generation

The Wing's native protocol only deals with numeric property IDs. To map human-readable names (like `/ch/1/fader`) to IDs, the library embeds a large static HashMap in `propmap.rs` — about 78k entries adding ~1MB to the binary and using a few MB of RAM when loaded. It's lazily initialized on first access (a few ms on a modern CPU).

If you don't need name lookups and want a smaller binary, copy `empty-propmap.rs` over `propmap.rs`. Note this will break some utility program functionality. You can also use `empty-propmap.rs` to recover if `propmap.rs` gets corrupted.

**The included `propmap.rs` was generated from a Wing Compact running firmware 3.0.5.**

### The dynamic nature of Wing properties

The Wing's property tree is **dynamic** — child nodes change depending on parent values. For example, if FX1 has type "NONE", `/fx/1` has only a few children. But loading an effect (e.g. setting type to "EXTERNAL") causes new properties like `/fx/1/trim` to appear.

The key dynamic property is `mdl` (model), found on FX slots, GATE slots, EQ slots, etc. Any node subtree containing a `mdl` StringEnum property will have different children depending on which model is selected.

### How wingschema handles dynamic properties

When `wingschema` encounters a `mdl` StringEnum, it:

1. Iterates through **every possible `mdl` value** in the enum
2. **Writes each value to the live device** (via `set_string`) to make those child properties appear
3. Queries the resulting child node schema for that model
4. Repeats for the next model value

This means running `wingschema` **destructively modifies your mixer's configuration**. The tool warns about this and requires typing "yes" to continue. You must save a snapshot beforehand and restore it after.

Only properties with a `mdl` StringEnum are known to be dynamic. If other dynamic properties are discovered, `wingschema` would need updating.

### Dynamic property paths in the propmap

Because of this `mdl` cycling, the property map contains paths for **all** possible models, even though only one model's properties exist on the device at any given time:
- `/fx/1/Chorus/rate` — properties specific to the Chorus FX model
- `/fx/1/Delay/time` — properties specific to the Delay FX model
- etc.

### Regenerating the propmap

Run `wingschema` against a live Wing device. It outputs two files:
- `propmap.rs` — copy to `src/` to update the library's built-in mapping
- `propmap.jsonl` — JSON Lines reference file for human inspection

## Key Concepts

- **Property tree**: The Wing exposes all parameters as a tree of nodes, each with a numeric ID and a path name (e.g. `/ch/1/fader`). The `propmap` maps names to IDs at compile time.
- **Blocking reads**: `read()` and `read_meters()` block until data arrives. Designed to be called from dedicated threads.
- **Unsolicited data**: The Wing pushes property changes whenever someone touches the physical mixer or another client writes a value. Your `read()` loop will see these interleaved with responses to your own requests.
- **Keep-alive**: Connections die if not kept alive. Call `keep_alive()` every ~7s for data, `keep_alive_meters()` every ~3s for meters.
- **Thread safety**: `WingConsole` is `Clone` and uses `Arc<Mutex>` internally. Separate threads for reading data, reading meters, and writing is the expected pattern.

## Protocol Quick Reference

The Wing supports 3 protocols: Native (binary), OSC, and Discovery. **libwing implements Native and Discovery only.** All three are documented in `Wing-Remote-Protocols.pdf` (by Patrick-Gilles Maillot, checked into this repo).

- TCP port 2222, binary protocol
- `0xdf` = message boundary / escape prefix; `0xde` = escape byte for literal `0xdf` in data
- `0xd1` = initial handshake byte sent on connect
- Property IDs are 32-bit integers encoded with the escape mechanism
- Discovery: UDP broadcast `"WING?"` to `255.255.255.255:2222`, response is CSV: `WING,ip,name,model,serial,firmware`

## Editing Guidelines

- `propmap.rs` is auto-generated by `wingschema` — don't edit by hand
- `ffi.rs` must stay in sync with `libwing.h` — update both together
- The protocol encoding/decoding in `console.rs` is subtle (escape sequences, channel state machine) — read carefully before modifying
