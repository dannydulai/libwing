#ifndef WING_C_API_H
#define WING_C_API_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>

// Node types
typedef enum {
    NODE_TYPE_NONE = 0,
    NODE_TYPE_FLOAT = 1,
    NODE_TYPE_INTEGER = 2,
    NODE_TYPE_STRING = 3,
    NODE_TYPE_BOOLEAN = 4,
    NODE_TYPE_ENUM = 5,
    NODE_TYPE_ACTION = 6,
    NODE_TYPE_COLOR = 7,
    NODE_TYPE_TIME = 8,
    NODE_TYPE_DATE = 9,
    NODE_TYPE_DATETIME = 10
} NodeType;

// Node units
typedef enum {
    NODE_UNIT_NONE = 0,
    NODE_UNIT_PERCENT = 1,
    NODE_UNIT_DB = 2,
    NODE_UNIT_HZ = 3,
    NODE_UNIT_SECONDS = 4,
    NODE_UNIT_DEGREES = 5,
    NODE_UNIT_METERS = 6,
    NODE_UNIT_KELVIN = 7,
    NODE_UNIT_VOLTS = 8,
    NODE_UNIT_AMPERES = 9,
    NODE_UNIT_WATTS = 10
} NodeUnit;

// Opaque handle types
typedef struct WingConsole_t* WingConsoleHandle;
typedef struct NodeData_t* NodeDataHandle;
typedef struct NodeDefinition_t* NodeDefinitionHandle;

// Discovery info structure
typedef struct {
    char ip[64];
    char name[64];
    char model[64];
    char serial[64];
    char firmware[64];
} WingDiscoveryInfo;

// Function declarations
WingConsoleHandle wing_console_connect(const char* ip);
void wing_console_close(WingConsoleHandle console);
void wing_console_read(WingConsoleHandle console);

void wing_console_set_string(WingConsoleHandle console, uint32_t id, const char* value);
void wing_console_set_float(WingConsoleHandle console, uint32_t id, float value);
void wing_console_set_int(WingConsoleHandle console, uint32_t id, int value);

void wing_console_request_node_definition(WingConsoleHandle console, uint32_t id);
void wing_console_request_node_data(WingConsoleHandle console, uint32_t id);

// Discovery functions
int wing_console_discover(WingDiscoveryInfo* info_array, size_t max_count, int stop_on_first);

// Callback function types
typedef void (*WingRequestEndCallback)(void* user_data);
typedef void (*WingNodeDefinitionCallback)(NodeDefinitionHandle def, void* user_data);
typedef void (*WingNodeDataCallback)(uint32_t id, NodeDataHandle data, void* user_data);

// Callback setting functions
void wing_console_set_request_end_callback(WingConsoleHandle console, WingRequestEndCallback callback, void* user_data);
void wing_console_set_node_definition_callback(WingConsoleHandle console, WingNodeDefinitionCallback callback, void* user_data);
void wing_console_set_node_data_callback(WingConsoleHandle console, WingNodeDataCallback callback, void* user_data);

// Node definition functions
NodeType wing_node_definition_get_type(NodeDefinitionHandle def);
NodeUnit wing_node_definition_get_unit(NodeDefinitionHandle def);
int wing_node_definition_is_read_only(NodeDefinitionHandle def);
uint32_t wing_node_definition_name_to_id(const char* name);
int wing_node_definition_id_to_name(uint32_t id, char* buffer, size_t buffer_size);

// Node data functions
// NodeDataHandle wing_node_data_create();
// void wing_node_data_destroy(NodeDataHandle data);
int wing_node_data_get_string(NodeDataHandle data, char* buffer, size_t buffer_size);
float wing_node_data_get_float(NodeDataHandle data);
int wing_node_data_get_int(NodeDataHandle data);
int wing_node_data_has_string(NodeDataHandle data);
int wing_node_data_has_float(NodeDataHandle data);
int wing_node_data_has_int(NodeDataHandle data);

#ifdef __cplusplus
}
#endif

#endif // WING_C_API_H
