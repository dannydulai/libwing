#include "wing_c_api.h"
#include "WingConsole.h"
#include "WingNode.h"
#include <cstring>

struct _wing_console_t { WingConsole console; };
struct _wing_discover_t { std::vector<DiscoveryInfo> info; };
struct _node_data_t { NodeData data; };
struct _node_definition_t { NodeDefinition def; };

wing_discover_t
wing_discover_scan(int stop_on_first)
{
    std::vector<DiscoveryInfo> results = WingConsole::scan(stop_on_first != 0);
    wing_discover_t discover_handle = new _wing_discover_t();
    discover_handle->info = results;
    return discover_handle;
}

void
wing_discover_destroy(wing_discover_t discover_handle)
{
    delete discover_handle;
}

int
wing_discover_count(wing_discover_t discover_handle)
{
    return discover_handle ? discover_handle->info.size() : 0;
}

const char *
wing_discover_get_ip(wing_discover_t discover_handle, int index)
{
    return discover_handle ? discover_handle->info[index].ip.c_str() : nullptr;
}

const char *
wing_discover_get_name(wing_discover_t discover_handle, int index)
{
    return discover_handle ? discover_handle->info[index].name.c_str() : nullptr;
}

const char *
wing_discover_get_model(wing_discover_t discover_handle, int index)
{
    return discover_handle ? discover_handle->info[index].model.c_str() : nullptr;
}

const char *
wing_discover_get_serial(wing_discover_t discover_handle, int index)
{
    return discover_handle ? discover_handle->info[index].serial.c_str() : nullptr;
}

const char *
wing_discover_get_firmware(wing_discover_t discover_handle, int index)
{
    return discover_handle ? discover_handle->info[index].firmware.c_str() : nullptr;
}

wing_console_t
wing_console_connect(const char *ip)
{
    try {
        wing_console_t console_handle = new _wing_console_t();
        console_handle->console = WingConsole::connect(ip);
        return console_handle;
    } catch (...) {
        return nullptr;
    }
}

void
wing_console_set_request_end_callback(wing_console_t console_handle,
                                      WingRequestEndCallback cb,
                                      void *user_data)
{
    if (console_handle) {
        console_handle->console.onRequestEnd = [cb, user_data]() {
            cb(user_data);
        };
    }
}

void
wing_console_set_node_definition_callback( wing_console_t console_handle,
                                           WingNodeDefinitionCallback cb,
                                           void *user_data)
{
    if (console_handle) {
        console_handle->console.onNodeDefinition = [cb,
        user_data](NodeDefinition def) {
            cb(new _node_definition_t{def}, user_data);
        };
    }
}

void
wing_console_set_node_data_callback(wing_console_t console_handle,
                                    WingNodeDataCallback cb,
                                    void *user_data)
{
    if (console_handle) {
        console_handle->console.onNodeData = [cb, user_data](uint32_t id, NodeData data) {
            cb(id, new _node_data_t{data}, user_data);
        };
    }
}

void
wing_console_destroy(wing_console_t console_handle)
{
    console_handle->console.close();
    delete console_handle;
}

void
wing_console_read(wing_console_t console_handle)
{
    if (console_handle) {
        console_handle->console.read();
    }
}

void
wing_console_set_string(wing_console_t console_handle,
                        uint32_t id,
                        const char *value)
{
    if (console_handle && value) {
        console_handle->console.setString(id, std::string(value));
    }
}

void
wing_console_set_float(wing_console_t console_handle,
                       uint32_t id,
                       float value)
{
    if (console_handle) {
        console_handle->console.setFloat(id, value);
    }
}

void
wing_console_set_int(wing_console_t console_handle,
                     uint32_t id,
                     int value)
{
    if (console_handle) {
        console_handle->console.setInt(id, value);
    }
}

void
wing_console_request_node_definition(wing_console_t console_handle,
                                     uint32_t id)
{
    if (console_handle) {
        console_handle->console.requestNodeDefinition(id);
    }
}

void
wing_console_request_node_data(wing_console_t console_handle,
                               uint32_t id)
{
    if (console_handle) {
        console_handle->console.requestNodeData(id);
    }
}

void
wing_node_definition_destroy(node_definition_t def_handle)
{
    delete def_handle;
}

node_type_t
wing_node_definition_get_type(node_definition_t def)
{
    return def ? (node_type_t)def->def.getType() : NODE_TYPE_NODE;
}

node_unit_t
wing_node_definition_get_unit(node_definition_t def)
{
    return def ? (node_unit_t)def->def.getUnit() : NODE_UNIT_NONE;
}

int
wing_node_definition_is_read_only(node_definition_t def)
{
    return def ? (def->def.isReadOnly() ? 1 : 0) : 0;
}

uint32_t
wing_node_name_to_id(const char* name)
{
    return name ? NodeDefinition::nodeNameToId(std::string(name)) : 0;
}

void
wing_node_id_to_name(uint32_t id, char* buffer, size_t buffer_size)
{
    std::string name = NodeDefinition::nodeIdToName(id);
    if (name.empty()) {
        buffer[0] = '\0';
    } else {
        strncpy(buffer, name.c_str(), buffer_size - 1);
        buffer[buffer_size - 1] = '\0';
    }
}

void
wing_node_data_destroy(node_data_t data_handle)
{
    delete data_handle;
}
void
wing_node_data_get_string(node_data_t data, char* buffer, size_t buffer_size)
{
    std::string str = data->data.getString();
    strncpy(buffer, str.c_str(), buffer_size - 1);
    buffer[buffer_size - 1] = '\0';
}

float
wing_node_data_get_float(node_data_t data)
{
    return data->data.getFloat();
}

int wing_node_data_get_int(node_data_t data) {
    return data->data.getInt();
}

int
wing_node_data_has_string(node_data_t data)
{
    return (data->data.hasString() ? 1 : 0);
}

int
wing_node_data_has_float(node_data_t data)
{
    return (data->data.hasFloat() ? 1 : 0);
}

int
wing_node_data_has_int(node_data_t data)
{
    return (data->data.hasInt() ? 1 : 0);
}

uint32_t
wing_node_definition_get_parent_id(node_definition_t def)
{
    return def->def.parentId;
}

uint32_t
wing_node_definition_get_id(node_definition_t def)
{
    return def->def.id;
}

uint16_t
wing_node_definition_get_index(node_definition_t def)
{
    return def->def.index;
}

void
wing_node_definition_get_name(node_definition_t def,
                              char* buffer,
                              size_t buffer_size)
{
    strncpy(buffer, def->def.name.c_str(), buffer_size - 1);
    buffer[buffer_size - 1] = '\0';
}

void
wing_node_definition_get_long_name(node_definition_t def,
                                   char* buffer,
                                   size_t buffer_size)
{
    strncpy(buffer, def->def.longName.c_str(), buffer_size - 1);
    buffer[buffer_size - 1] = '\0';
}

float
wing_node_definition_get_min_float(node_definition_t def)
{
    return def->def.minFloat;
}

float
wing_node_definition_get_max_float(node_definition_t def)
{
    return def->def.maxFloat;
}

uint32_t
wing_node_definition_get_steps(node_definition_t def)
{
    return def->def.steps;
}

int32_t
wing_node_definition_get_min_int(node_definition_t def)
{
    return def->def.minInt;
}

int32_t
wing_node_definition_get_max_int(node_definition_t def)
{
    return def->def.maxInt;
}

uint16_t
wing_node_definition_get_max_string_len(node_definition_t def)
{
    return def->def.maxStringLen;
}

size_t
wing_node_definition_get_string_enum_count(node_definition_t def)
{
    return def->def.stringEnum.size();
}

void
wing_node_definition_get_string_enum_item(node_definition_t def,
                                          size_t index,
                                          char *item_buffer,
                                          size_t item_buffer_size,
                                          char *longitem_buffer,
                                          size_t longitem_buffer_size)
{
    const auto &item = def->def.stringEnum[index];
    strncpy(item_buffer, item.item.c_str(), item_buffer_size - 1);
    item_buffer[item_buffer_size - 1] = '\0';

    strncpy(longitem_buffer, item.longitem.c_str(), longitem_buffer_size - 1);
    longitem_buffer[longitem_buffer_size - 1] = '\0';
}

size_t
wing_node_definition_get_float_enum_count(node_definition_t def)
{
    return def->def.floatEnum.size();
}

void
wing_node_definition_get_float_enum_item(node_definition_t def,
                                         size_t index,
                                         float* item_value,
                                         char* longitem_buffer,
                                         size_t longitem_buffer_size)
{
    const auto& item = def->def.floatEnum[index];
    *item_value = item.item;

    strncpy(longitem_buffer, item.longitem.c_str(), longitem_buffer_size - 1);
    longitem_buffer[longitem_buffer_size - 1] = '\0';
}
