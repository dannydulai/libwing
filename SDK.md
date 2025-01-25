# LibWing SDK Documentation

LibWing is a C++ library for interfacing with Behringer Wing digital mixing consoles. It provides functionality for discovering Wing consoles on the network, connecting to them, and reading/writing console parameters.

## Basic Concepts

The Wing console exposes its functionality through a tree of nodes. Each node has:
- A unique numeric ID
- A hierarchical path name (like a filesystem path)
- A type (string, float, integer, enum, etc.)
- Optional min/max values and units
- Read/write or read-only access

## Getting Started

### Discovery
To find Wing consoles on your network:

```cpp
auto discovered = WingConsole::discover();
if (!discovered.empty()) {
    // Found at least one console
    auto firstConsole = discovered[0];
    std::cout << "Found console: " << firstConsole.name 
              << " at IP: " << firstConsole.ip << std::endl;
}
```

### Connecting
Once you have a console's IP address, you can connect to it:

```cpp
auto console = WingConsole::connect("192.168.1.100");
```

### Communication Model

The Wing console uses an asynchronous communication model:

1. You make requests using methods like `requestNodeDefinition()` and `requestNodeData()`
2. Responses come back through callback functions you register
3. The console also sends unsolicited updates when values change
4. All messages are received through the `read()` method, which you must call repeatedly

### Callbacks

Register callbacks to handle different types of messages:

```cpp
// Called when a node definition is received
console.onNodeDefinition = [](NodeDefinition node) {
    std::cout << "Got definition for node: " << node.name << std::endl;
};

// Called when node data/values are received
console.onNodeData = [](uint32_t id, NodeData data) {
    std::cout << "Node " << id << " value: " << data.getString() << std::endl;
};

// Called when a request is complete
console.onRequestEnd = []() {
    std::cout << "Request completed" << std::endl;
};
```

### Reading Data

To start receiving data:

```cpp
console.read();  // This blocks and processes incoming messages
```

### Writing Data

To change values on the console:

```cpp
// Set values by node ID
console.setString(nodeId, "value");
console.setFloat(nodeId, 0.5f);
console.setInt(nodeId, 42);
```

## Example Programs

### wingmon
The `wingmon` program demonstrates basic connection and monitoring:
- Discovers consoles on the network
- Connects to the first one found
- Monitors and displays value changes

### wingschema 
The `wingschema` program shows how to:
- Traverse the entire node tree
- Request and process node definitions
- Generate documentation of the console's parameter space
- Handle asynchronous responses systematically

## Node Types

Nodes can have these types:
- NODE_TYPE_NODE: A container for other nodes
- NODE_TYPE_LINEAR_FLOAT: Linear floating point value
- NODE_TYPE_LOGARITHMIC_FLOAT: Logarithmic floating point value
- NODE_TYPE_FADER_LEVEL: Special case for fader positions
- NODE_TYPE_INTEGER: Integer value
- NODE_TYPE_STRING_ENUM: String with predefined options
- NODE_TYPE_FLOAT_ENUM: Float with predefined options
- NODE_TYPE_STRING: Free-form string

## Units

Values can have these units:
- NODE_UNIT_NONE: No unit
- NODE_UNIT_DB: Decibels
- NODE_UNIT_PERCENT: Percentage
- NODE_UNIT_MILLISECONDS: Time in ms
- NODE_UNIT_HERTZ: Frequency
- NODE_UNIT_METERS: Distance
- NODE_UNIT_SECONDS: Time in seconds
- NODE_UNIT_OCTAVES: Musical octaves

## Best Practices

1. Always check discovery results before connecting
2. Set up callbacks before making requests
3. Handle both solicited and unsolicited messages
4. Keep track of request completion via onRequestEnd
5. Use node IDs for efficiency, paths for readability
6. Check node definitions for read-only status before writing
7. Respect min/max values and enumerated options

## Building

LibWing uses CMake for building. Basic build steps:

```bash
mkdir build
cd build
cmake ..
make
```
