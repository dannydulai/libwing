#include "wing_c_api.h"
#include "WingConsole.h"
#include "WingNode.h"
#include <cstring>

struct _wing_console_t { WingConsole console; };
struct _node_data_t { NodeData data; };
struct _node_definition_t { NodeDefinition def; };

wing_console_t wing_console_connect(const char* ip) {
    try {
        wing_console_t handle = new _wing_console_t();
        handle->console = WingConsole::connect(ip);
        return handle;
    } catch (...) {
        return nullptr;
    }
}

void wing_console_set_request_end_callback(wing_console_t console, WingRequestEndCallback callback, void* user_data) {
    if (console) {
        console->console.onRequestEnd = [callback, user_data]() {
            callback(user_data);
        };
    }
}

void wing_console_set_node_definition_callback(wing_console_t console, WingNodeDefinitionCallback callback, void* user_data) {
    if (console) {
        console->console.onNodeDefinition = [callback, user_data](NodeDefinition def) {
            node_definition_t def_handle = new _node_definition_t{def};
            callback(def_handle, user_data);
            delete def_handle;
        };
    }
}

void wing_console_set_node_data_callback(wing_console_t console, WingNodeDataCallback callback, void* user_data) {
    if (console) {
        console->console.onNodeData = [callback, user_data](uint32_t id, NodeData data) {
            node_data_t data_handle = new _node_data_t{data};
            callback(id, data_handle, user_data);
            delete data_handle;
        };
    }
}

void wing_console_close(wing_console_t console) {
    if (console) {
        delete console;
    }
}

void wing_console_read(wing_console_t console) {
    if (console) {
        console->console.read();
    }
}

void wing_console_set_string(wing_console_t console, uint32_t id, const char* value) {
    if (console && value) {
        console->console.setString(id, std::string(value));
    }
}

void wing_console_set_float(wing_console_t console, uint32_t id, float value) {
    if (console) {
        console->console.setFloat(id, value);
    }
}

void wing_console_set_int(wing_console_t console, uint32_t id, int value) {
    if (console) {
        console->console.setInt(id, value);
    }
}

void wing_console_request_node_definition(wing_console_t console, uint32_t id) {
    if (console) {
        console->console.requestNodeDefinition(id);
    }
}

void wing_console_request_node_data(wing_console_t console, uint32_t id) {
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

node_type_t wing_node_definition_get_type(node_definition_t def) {
    return def ? (node_type_t)def->def.getType() : NODE_TYPE_NODE;
}

node_unit_t wing_node_definition_get_unit(node_definition_t def) {
    return def ? (node_unit_t)def->def.getUnit() : NODE_UNIT_NONE;
}

int wing_node_definition_is_read_only(node_definition_t def) {
    return def ? (def->def.isReadOnly() ? 1 : 0) : 0;
}

uint32_t wing_node_definition_name_to_id(const char* name) {
    return name ? NodeDefinition::nodeNameToId(std::string(name)) : 0;
}

int wing_node_definition_id_to_name(uint32_t id, char* buffer, size_t buffer_size) {
    if (!buffer || buffer_size == 0) {
        return 0;
    }
    std::string name = NodeDefinition::nodeIdToName(id);
    if (name.empty()) {
        buffer[0] = '\0';
        return 0;
    }
    strncpy(buffer, name.c_str(), buffer_size - 1);
    buffer[buffer_size - 1] = '\0';
    return 1;
}

int wing_node_data_get_string(node_data_t data, char* buffer, size_t buffer_size) {
    if (!data || !buffer || buffer_size == 0) {
        return 0;
    }
    std::string str = data->data.getString();
    strncpy(buffer, str.c_str(), buffer_size - 1);
    buffer[buffer_size - 1] = '\0';
    return 1;
}

float wing_node_data_get_float(node_data_t data) {
    return data ? data->data.getFloat() : 0.0f;
}

int wing_node_data_get_int(node_data_t data) {
    return data ? data->data.getInt() : 0;
}

int wing_node_data_has_string(node_data_t data) {
    return data ? (data->data.hasString() ? 1 : 0) : 0;
}

int wing_node_data_has_float(node_data_t data) {
    return data ? (data->data.hasFloat() ? 1 : 0) : 0;
}

int wing_node_data_has_int(node_data_t data) {
    return data ? (data->data.hasInt() ? 1 : 0) : 0;
}
