#ifndef WING_C_API_H
#define WING_C_API_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>

// Opaque handle types
typedef struct WingConsole_t* WingConsoleHandle;
typedef struct NodeData_t* NodeDataHandle;

// Discovery info structure
typedef struct {
    char ip[64];
    char name[64];
    char model[64];
    char serial[64];
    char firmware[64];
} WingDiscoveryInfo;

// Node definition structure
typedef struct {
    uint32_t parentId;
    uint32_t id;
    uint16_t index;
    char name[64];
    char longname[128];
    uint16_t flags;
    float minFloat;
    float maxFloat;
    uint32_t steps;
    int32_t minInt;
    int32_t maxInt;
    uint16_t maxStringLen;
} WingNodeDefinition;

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

// Node data functions
NodeDataHandle wing_node_data_create();
void wing_node_data_destroy(NodeDataHandle data);
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
