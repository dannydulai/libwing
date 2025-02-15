#ifndef LIBWING_H
#define LIBWING_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle types
typedef struct WingDiscoveryInfo WingDiscoveryInfo;
typedef struct WingConsole WingConsole;
typedef struct Response Response;

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
WingDiscoveryInfo* wing_discover_scan                             (int stop_on_first);
void               wing_discover_destroy                          (WingDiscoveryInfo* handle);
int                wing_discover_count                            (const WingDiscoveryInfo* handle);
const char*        wing_discover_get_ip                           (const WingDiscoveryInfo* handle, int index);
const char*        wing_discover_get_name                         (const WingDiscoveryInfo* handle, int index);
const char*        wing_discover_get_model                        (const WingDiscoveryInfo* handle, int index);
const char*        wing_discover_get_serial                       (const WingDiscoveryInfo* handle, int index);
const char*        wing_discover_get_firmware                     (const WingDiscoveryInfo* handle, int index);

// Console functions
WingConsole*       wing_console_connect                           (const char* ip);
void               wing_console_destroy                           (WingConsole* handle);
Response*          wing_console_read                              (WingConsole* handle);
int                wing_console_set_string                        (WingConsole* handle, int32_t id, const char* value);
int                wing_console_set_float                         (WingConsole* handle, int32_t id, float value);
int                wing_console_set_int                           (WingConsole* handle, int32_t id, int value);
int                wing_console_request_node_definition           (WingConsole* handle, int32_t id);
int                wing_console_request_node_data                 (WingConsole* handle, int32_t id);

// Response functions
void               wing_response_destroy                          (Response* handle);
WingResponseType   wing_response_get_type                         (const Response* handle);

// Node data functions
const char*        wing_node_data_get_string                      (const Response* handle);
float              wing_node_data_get_float                       (const Response* handle);
int                wing_node_data_get_int                         (const Response* handle);
int                wing_node_data_has_string                      (const Response* handle);
int                wing_node_data_has_float                       (const Response* handle);
int                wing_node_data_has_int                         (const Response* handle);

// Node definition functions
int32_t            wing_node_definition_get_parent_id             (const Response* handle);
int32_t            wing_node_definition_get_id                    (const Response* handle);
uint16_t           wing_node_definition_get_index                 (const Response* handle);
WingNodeType       wing_node_definition_get_type                  (const Response* handle);
WingNodeUnit       wing_node_definition_get_unit                  (const Response* handle);
const char*        wing_node_definition_get_name                  (const Response* handle);
const char*        wing_node_definition_get_long_name             (const Response* handle);
int                wing_node_definition_is_read_only              (const Response* handle);
int                wing_node_definition_get_min_float             (const Response* handle, float* ret);
int                wing_node_definition_get_max_float             (const Response* handle, float* ret);
int                wing_node_definition_get_steps                 (const Response* handle, int* ret);
int                wing_node_definition_get_min_int               (const Response* handle, int* ret);
int                wing_node_definition_get_max_int               (const Response* handle, int* ret);
int                wing_node_definition_get_max_string_len        (const Response* handle, int* ret);
int                wing_node_definition_get_string_enum_count     (const Response* handle);
int                wing_node_definition_get_float_enum_count      (const Response* handle);
int                wing_node_definition_get_float_enum_item       (const Response* handle, int index, float* ret);
int                wing_node_definition_get_float_enum_long_item  (const Response* handle, int index, const char** ret);
int                wing_node_definition_get_string_enum_item      (const Response* handle, int index, const char** ret);
int                wing_node_definition_get_string_enum_long_item (const Response* handle, int index, const char** ret);

// Utility functions
int                wing_name_to_id                                (const char* name, int32_t* out_id);

    // you must call this to free the memory of any string returned by the library
void               wing_string_destroy                            (const char* handle);

#ifdef __cplusplus
}
#endif

#endif /* LIBWING_H */
