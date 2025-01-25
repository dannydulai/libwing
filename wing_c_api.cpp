#include "wing_c_api.h"
#include "WingConsole.h"
#include "WingNode.h"
#include <cstring>

struct WingConsole_t {
    WingConsole console;
    WingRequestEndCallback request_end_cb;
    WingNodeDefinitionCallback node_def_cb;
    WingNodeDataCallback node_data_cb;
    void* request_end_user_data;
    void* node_def_user_data;
    void* node_data_user_data;
    
    WingConsole_t() : request_end_cb(nullptr), node_def_cb(nullptr), node_data_cb(nullptr),
                      request_end_user_data(nullptr), node_def_user_data(nullptr), node_data_user_data(nullptr) {}
};

struct NodeData_t {
    NodeData data;
};

WingConsoleHandle wing_console_connect(const char* ip) {
    try {
        WingConsoleHandle handle = new WingConsole_t();
        handle->console = WingConsole::connect(ip);
        
        // Set up C++ callbacks that will forward to C callbacks
        handle->console.onRequestEnd = [handle]() {
            if (handle->request_end_cb) {
                handle->request_end_cb(handle->request_end_user_data);
            }
        };
        
        handle->console.onNodeDefinition = [handle](NodeDefinition def) {
            if (handle->node_def_cb) {
                WingNodeDefinition c_def = {}; // Convert NodeDefinition to C struct
                // TODO: Fill in c_def fields from def
                handle->node_def_cb(&c_def, handle->node_def_user_data);
            }
        };
        
        handle->console.onNodeData = [handle](uint32_t id, NodeData data) {
            if (handle->node_data_cb) {
                NodeDataHandle data_handle = new NodeData_t{data};
                handle->node_data_cb(id, data_handle, handle->node_data_user_data);
                delete data_handle;
            }
        };
        
        return handle;
    } catch (...) {
        return nullptr;
    }
}

void wing_console_set_request_end_callback(WingConsoleHandle console, WingRequestEndCallback callback, void* user_data) {
    if (console) {
        console->request_end_cb = callback;
        console->request_end_user_data = user_data;
    }
}

void wing_console_set_node_definition_callback(WingConsoleHandle console, WingNodeDefinitionCallback callback, void* user_data) {
    if (console) {
        console->node_def_cb = callback;
        console->node_def_user_data = user_data;
    }
}

void wing_console_set_node_data_callback(WingConsoleHandle console, WingNodeDataCallback callback, void* user_data) {
    if (console) {
        console->node_data_cb = callback;
        console->node_data_user_data = user_data;
    }
}

void wing_console_close(WingConsoleHandle console) {
    if (console) {
        console->console.close();
        delete console;
    }
}

void wing_console_read(WingConsoleHandle console) {
    if (console) {
        console->console.read();
    }
}

void wing_console_set_string(WingConsoleHandle console, uint32_t id, const char* value) {
    if (console && value) {
        console->console.setString(id, std::string(value));
    }
}

void wing_console_set_float(WingConsoleHandle console, uint32_t id, float value) {
    if (console) {
        console->console.setFloat(id, value);
    }
}

void wing_console_set_int(WingConsoleHandle console, uint32_t id, int value) {
    if (console) {
        console->console.setInt(id, value);
    }
}

void wing_console_request_node_definition(WingConsoleHandle console, uint32_t id) {
    if (console) {
        console->console.requestNodeDefinition(id);
    }
}

void wing_console_request_node_data(WingConsoleHandle console, uint32_t id) {
    if (console) {
        console->console.requestNodeData(id);
    }
}

int wing_console_discover(WingDiscoveryInfo* info_array, size_t max_count, int stop_on_first) {
    if (!info_array || max_count == 0) {
        return 0;
    }

    std::vector<DiscoveryInfo> results = WingConsole::discover(stop_on_first != 0);
    size_t count = std::min(results.size(), max_count);

    for (size_t i = 0; i < count; i++) {
        strncpy(info_array[i].ip, results[i].ip.c_str(), sizeof(info_array[i].ip) - 1);
        strncpy(info_array[i].name, results[i].name.c_str(), sizeof(info_array[i].name) - 1);
        strncpy(info_array[i].model, results[i].model.c_str(), sizeof(info_array[i].model) - 1);
        strncpy(info_array[i].serial, results[i].serial.c_str(), sizeof(info_array[i].serial) - 1);
        strncpy(info_array[i].firmware, results[i].firmware.c_str(), sizeof(info_array[i].firmware) - 1);
    }

    return static_cast<int>(count);
}

NodeDataHandle wing_node_data_create() {
    return new NodeData_t();
}

void wing_node_data_destroy(NodeDataHandle data) {
    delete data;
}

int wing_node_data_get_string(NodeDataHandle data, char* buffer, size_t buffer_size) {
    if (!data || !buffer || buffer_size == 0) {
        return 0;
    }
    std::string str = data->data.getString();
    strncpy(buffer, str.c_str(), buffer_size - 1);
    buffer[buffer_size - 1] = '\0';
    return 1;
}

float wing_node_data_get_float(NodeDataHandle data) {
    return data ? data->data.getFloat() : 0.0f;
}

int wing_node_data_get_int(NodeDataHandle data) {
    return data ? data->data.getInt() : 0;
}

int wing_node_data_has_string(NodeDataHandle data) {
    return data ? (data->data.hasString() ? 1 : 0) : 0;
}

int wing_node_data_has_float(NodeDataHandle data) {
    return data ? (data->data.hasFloat() ? 1 : 0) : 0;
}

int wing_node_data_has_int(NodeDataHandle data) {
    return data ? (data->data.hasInt() ? 1 : 0) : 0;
}
