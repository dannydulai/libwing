#ifndef WING_C_API_H
#define WING_C_API_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>

// Node types
typedef enum {
    NODE_TYPE_NODE              = 0,
    NODE_TYPE_LINEAR_FLOAT      = 1,
    NODE_TYPE_LOGARITHMIC_FLOAT = 2,
    NODE_TYPE_FADER_LEVEL       = 3,
    NODE_TYPE_INTEGER           = 4,
    NODE_TYPE_STRING_ENUM       = 5,
    NODE_TYPE_FLOAT_ENUM        = 6,
    NODE_TYPE_STRING            = 7
} node_type_t;

// Node units
typedef enum {
    NODE_UNIT_NONE         = 0,
    NODE_UNIT_DB           = 1,
    NODE_UNIT_PERCENT      = 2,
    NODE_UNIT_MILLISECONDS = 3,
    NODE_UNIT_HERTZ        = 4,
    NODE_UNIT_METERS       = 5,
    NODE_UNIT_SECONDS      = 6,
    NODE_UNIT_OCTAVES      = 7,
} node_unit_t;

// Opaque handle types
typedef struct _wing_discover_t*   wing_discover_t;
typedef struct _wing_console_t*    wing_console_t;
typedef struct _node_definition_t* node_definition_t;
typedef struct _node_data_t*       node_data_t;

wing_discover_t wing_discover_scan                 (int stop_on_first);
void            wing_discover_destroy              (wing_discover_t);
int             wing_discover_count                (wing_discover_t);
const char *    wing_discover_get_ip               (wing_discover_t, int index);
const char *    wing_discover_get_name             (wing_discover_t, int index);
const char *    wing_discover_get_model            (wing_discover_t, int index);
const char *    wing_discover_get_serial           (wing_discover_t, int index);
const char *    wing_discover_get_firmware         (wing_discover_t, int index);

wing_console_t wing_console_connect                (const char* ip);
void           wing_console_destroy                (wing_console_t);
void           wing_console_read                   (wing_console_t);

void           wing_console_set_string             (wing_console_t, uint32_t id, const char* value);
void           wing_console_set_float              (wing_console_t, uint32_t id, float value);
void           wing_console_set_int                (wing_console_t, uint32_t id, int value);
               
void           wing_console_request_node_definition(wing_console_t, uint32_t id);
void           wing_console_request_node_data      (wing_console_t, uint32_t id);

// Callback function types
typedef void (*WingRequestEndCallback)(void* user_data);
typedef void (*WingNodeDefinitionCallback)(node_definition_t def, void* user_data);
typedef void (*WingNodeDataCallback)(uint32_t id, node_data_t data, void* user_data);

// Callback setting functions
void     wing_console_set_request_end_callback    (wing_console_t, WingRequestEndCallback callback, void* user_data);
void     wing_console_set_node_definition_callback(wing_console_t, WingNodeDefinitionCallback callback, void* user_data);
void     wing_console_set_node_data_callback      (wing_console_t, WingNodeDataCallback callback, void* user_data);

int         wing_node_init_map                         (const char* path_to_map_file);
uint32_t    wing_node_name_to_id                       (const char* name);
void        wing_node_id_to_name                       (uint32_t id, char* buffer, size_t buffer_size);

// Node definition functions
void        wing_node_definition_destroy               (node_definition_t);
node_type_t wing_node_definition_get_type              (node_definition_t);
node_unit_t wing_node_definition_get_unit              (node_definition_t);
int         wing_node_definition_is_read_only          (node_definition_t);
uint32_t    wing_node_definition_get_parent_id         (node_definition_t);
uint32_t    wing_node_definition_get_id                (node_definition_t);
uint16_t    wing_node_definition_get_index             (node_definition_t);
void        wing_node_definition_get_name              (node_definition_t, char* buffer, size_t buffer_size);
void        wing_node_definition_get_long_name         (node_definition_t, char* buffer, size_t buffer_size);
float       wing_node_definition_get_min_float         (node_definition_t);
float       wing_node_definition_get_max_float         (node_definition_t);
uint32_t    wing_node_definition_get_steps             (node_definition_t);
int32_t     wing_node_definition_get_min_int           (node_definition_t);
int32_t     wing_node_definition_get_max_int           (node_definition_t);
uint16_t    wing_node_definition_get_max_string_len    (node_definition_t);

// Enum accessors
size_t      wing_node_definition_get_string_enum_count (node_definition_t);
void        wing_node_definition_get_string_enum_item  (node_definition_t, size_t index, 
                                                        char* item_buffer, size_t item_buffer_size,
                                                        char* longitem_buffer, size_t longitem_buffer_size);
size_t      wing_node_definition_get_float_enum_count  (node_definition_t);
void        wing_node_definition_get_float_enum_item   (node_definition_t, size_t index,
                                                        float* item_value,
                                                        char* longitem_buffer, size_t longitem_buffer_size);

// Node data functions
// NodeDataHandle wing_node_data_create();
// void wing_node_data_destroy(NodeDataHandle data);
void  wing_node_data_destroy   (node_data_t);
void  wing_node_data_get_string(node_data_t, char* buffer, size_t buffer_size);
float wing_node_data_get_float (node_data_t);
int   wing_node_data_get_int   (node_data_t);
int   wing_node_data_has_string(node_data_t);
int   wing_node_data_has_float (node_data_t);
int   wing_node_data_has_int   (node_data_t);

#ifdef __cplusplus
}
#endif

#endif // WING_C_API_H
