# Wing.Fader API Documentation

**📖 The complete API documentation has been moved to the module documentation.**

To view the full documentation, use:

```elixir
# In IEx
h Wing.Fader

# Or generate HTML docs
mix docs
```

## Quick Reference

```elixir
# Connect to console
console = Wing.connect_with_host("192.168.1.100")

# Set faders
Wing.Fader.set_channel_fader(console, 1, -10.0)
Wing.Fader.set_bus_fader(console, 1, -5.0)
Wing.Fader.set_main_fader(console, :lr, 0.0)
Wing.Fader.set_main_fader(console, 1, -8.0)  # Matrix 1

# Subscribe to changes
Wing.Fader.subscribe_channel_fader(console, 1, self())
Wing.Fader.subscribe_main_fader(console, :lr, self())

# Receive notifications
receive do
  {:fader_changed, :channel, 1, value} -> IO.puts("Ch1: #{value} dB")
  {:fader_changed, :main, :lr, value} -> IO.puts("LR: #{value} dB")
end
```

For complete documentation including examples, error handling, and implementation details, see the module documentation with `h Wing.Fader` in IEx.

## Features Implemented

### ✅ Fader Control Functions
- **Channel faders**: `set_channel_fader(console, channel, db_value)`
- **Bus faders**: `set_bus_fader(console, bus, db_value)` 
- **Main outputs**: `set_main_fader(console, :lr | :mono | matrix_num, db_value)`

### ✅ Real-time Subscription System
- **Channel monitoring**: `subscribe_channel_fader(console, channel, subscriber_pid)`
- **Bus monitoring**: `subscribe_bus_fader(console, bus, subscriber_pid)`
- **Main monitoring**: `subscribe_main_fader(console, :lr | :mono | matrix_num, subscriber_pid)`

### ✅ Robust Error Handling
- Input validation with descriptive error messages
- Graceful handling of invalid property paths
- Connection error management

### ✅ Automatic Process Management
- GenServer-based subscription management
- Automatic cleanup when subscriber processes die
- Background property monitoring threads

## Supported Fader Types

| Type | Range | Property Path | Example |
|------|-------|---------------|---------|
| Channels | 1-40 | `/ch/{n}/fdr` | `set_channel_fader(console, 1, -10.0)` |
| Buses | 1-16 | `/bus/{n}/fdr` | `set_bus_fader(console, 1, -5.0)` |
| Main LR | `:lr` | `/main/1/fdr` | `set_main_fader(console, :lr, 0.0)` |
| Main Mono | `:mono` | `/main/2/fdr` | `set_main_fader(console, :mono, -3.0)` |
| Matrix | 1-6 | `/mtx/{n}/fdr` | `set_main_fader(console, 1, -8.0)` |

## Message Format

Subscriber processes receive real-time fader change notifications:

```elixir
{:fader_changed, fader_type, identifier, new_value_db}
```

Where:
- `fader_type`: `:channel`, `:bus`, or `:main`
- `identifier`: Channel/bus number or main type (`:lr`, `:mono`, or matrix number)
- `new_value_db`: New fader value in decibels (float)

## Testing Status

- ✅ **Functional tests**: All basic operations working
- ✅ **Error handling**: Invalid inputs properly caught
- ✅ **Console integration**: Successfully tested with Wing console at IP 10.10.14.85
- ✅ **Property paths**: Verified correct property mapping for all fader types
- ✅ **Subscription system**: Real-time change notifications working

## Implementation Details

### Property Path Mapping
The module correctly maps user-friendly fader types to Wing property paths:
- Main LR → `/main/1/fdr` (corrected from `/main/st/fdr`)
- Main Mono → `/main/2/fdr` (corrected from `/main/m/fdr`)
- Matrix outputs → `/mtx/{1-6}/fdr` (confirmed working)

### NIF Integration
Uses the underlying Rust NIFs for efficient communication:
- `Wing.connect_with_host/1` - Console connection
- `Wing.name_to_id/1` - Property path to ID conversion
- `Wing.set_float/3` - Setting fader values
- `Wing.start_property_thread/3` - Real-time change monitoring

### Process Architecture
- **Client API**: Simple functions for common operations
- **GenServer Manager**: Handles subscriptions and process monitoring
- **Background Threads**: Rust-based property change monitoring
- **Message Translation**: Converts raw property changes to meaningful notifications

This implementation provides a complete, production-ready solution for Wing console fader control and monitoring.
