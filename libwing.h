#ifndef LIBWING_H
#define LIBWING_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle types
typedef struct WingDiscoveryInfoHandle WingDiscoveryInfoHandle;
typedef struct WingConsoleHandle WingConsoleHandle;
typedef struct ResponseHandle ResponseHandle;

// Enums
typedef enum {
    WING_RESPONSE_END = 0,
    WING_RESPONSE_NODE_DEFINITION = 1,
    WING_RESPONSE_NODE_DATA = 2
} WingResponseType;

typedef enum {
    WING_NODE_TYPE_NODE = 0,
    WING_NODE_TYPE_LINEAR_FLOAT = 1,
    WING_NODE_TYPE_LOGARITHMIC_FLOAT = 2,
    WING_NODE_TYPE_FADER_LEVEL = 3,
    WING_NODE_TYPE_INTEGER = 4,
    WING_NODE_TYPE_STRING_ENUM = 5,
    WING_NODE_TYPE_FLOAT_ENUM = 6,
    WING_NODE_TYPE_STRING = 7
} WingNodeType;

typedef enum {
    WING_NODE_UNIT_NONE = 0,
    WING_NODE_UNIT_DB = 1,
    WING_NODE_UNIT_PERCENT = 2,
    WING_NODE_UNIT_MILLISECONDS = 3,
    WING_NODE_UNIT_HERTZ = 4,
    WING_NODE_UNIT_METERS = 5,
    WING_NODE_UNIT_SECONDS = 6,
    WING_NODE_UNIT_OCTAVES = 7
} WingNodeUnit;

// Discovery functions
WingDiscoveryInfoHandle* wing_discover_scan(int stop_on_first);
void wing_discover_destroy(WingDiscoveryInfoHandle* handle);
int wing_discover_count(const WingDiscoveryInfoHandle* handle);
const char* wing_discover_get_ip(const WingDiscoveryInfoHandle* handle, int index);
const char* wing_discover_get_name(const WingDiscoveryInfoHandle* handle, int index);
const char* wing_discover_get_model(const WingDiscoveryInfoHandle* handle, int index);
const char* wing_discover_get_serial(const WingDiscoveryInfoHandle* handle, int index);
const char* wing_discover_get_firmware(const WingDiscoveryInfoHandle* handle, int index);

// Console functions
WingConsoleHandle* wing_console_connect(const char* ip);
void wing_console_destroy(WingConsoleHandle* handle);
ResponseHandle* wing_console_read(WingConsoleHandle* handle);
int wing_console_set_string(WingConsoleHandle* handle, int32_t id, const char* value);
int wing_console_set_float(WingConsoleHandle* handle, int32_t id, float value);
int wing_console_set_int(WingConsoleHandle* handle, int32_t id, int value);
int wing_console_request_node_definition(WingConsoleHandle* handle, int32_t id);
int wing_console_request_node_data(WingConsoleHandle* handle, int32_t id);

// Response functions
void wing_response_destroy(ResponseHandle* handle);
WingResponseType wing_response_get_type(const ResponseHandle* handle);
int32_t wing_response_get_node_id(const ResponseHandle* handle);

// Node data functions
const char* wing_node_data_get_string(const ResponseHandle* handle);
float wing_node_data_get_float(const ResponseHandle* handle);
int wing_node_data_get_int(const ResponseHandle* handle);
int wing_node_data_has_string(const ResponseHandle* handle);
int wing_node_data_has_float(const ResponseHandle* handle);
int wing_node_data_has_int(const ResponseHandle* handle);

// Node definition functions
int32_t wing_node_definition_get_parent_id(const ResponseHandle* handle);
int32_t wing_node_definition_get_id(const ResponseHandle* handle);
uint16_t wing_node_definition_get_index(const ResponseHandle* handle);
WingNodeType wing_node_definition_get_type(const ResponseHandle* handle);
WingNodeUnit wing_node_definition_get_unit(const ResponseHandle* handle);
const char* wing_node_definition_get_name(const ResponseHandle* handle);
const char* wing_node_definition_get_long_name(const ResponseHandle* handle);
int wing_node_definition_is_read_only(const ResponseHandle* handle);
int wing_node_definition_get_min_float(const ResponseHandle* handle, float* ret);
int wing_node_definition_get_max_float(const ResponseHandle* handle, float* ret);
int wing_node_definition_get_steps(const ResponseHandle* handle, int* ret);
int wing_node_definition_get_min_int(const ResponseHandle* handle, int* ret);
int wing_node_definition_get_max_int(const ResponseHandle* handle, int* ret);
int wing_node_definition_get_max_string_len(const ResponseHandle* handle, int* ret);
size_t wing_node_definition_get_string_enum_count(const ResponseHandle* handle);
size_t wing_node_definition_get_float_enum_count(const ResponseHandle* handle);
int wing_node_definition_get_float_enum_item(const ResponseHandle* handle, size_t index, float* ret);
int wing_node_definition_get_float_enum_long_item(const ResponseHandle* handle, size_t index, char** ret);
int wing_node_definition_get_string_enum_item(const ResponseHandle* handle, size_t index, char** ret);
int wing_node_definition_get_string_enum_long_item(const ResponseHandle* handle, size_t index, char** ret);

// Utility functions
int32_t wing_name_to_id(const char* name);
const char* wing_id_to_name(int32_t id);
void wing_string_destroy(const char* handle);

#ifdef __cplusplus
}
#endif

#endif /* LIBWING_H */
