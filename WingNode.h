#ifndef __WINGNODE_H
#define __WINGNODE_H
#include <string>
#include <map>
#include <vector>

namespace WingNode {
    enum Unit {
        UNIT_NONE         = 0,
        UNIT_DB           = 1,
        UNIT_PERCENT      = 2,
        UNIT_MILLISECONDS = 3,
        UNIT_HERTZ        = 4,
        UNIT_METERS       = 5,
        UNIT_SECONDS      = 6,
        UNIT_OCTAVES      = 7,
    };
    enum Type {
        TYPE_NODE              = 0,
        TYPE_LINEAR_FLOAT      = 1,
        TYPE_LOGARITHMIC_FLOAT = 2,
        TYPE_FADER_LEVEL       = 3,
        TYPE_INTEGER           = 4,
        TYPE_STRING_ENUM       = 5,
        TYPE_FLOAT_ENUM        = 6,
        TYPE_STRING            = 7
    };
}

struct StringEnumItem {
    std::string item;
    std::string longitem;
};

struct FloatEnumItem {
    float       item;
    std::string longitem;
};

struct NodeDefinition {
private:
    uint32_t                    parentId;
    uint32_t                    id;
    uint16_t                    index;
    std::string                 name;
    std::string                 longname;
    uint16_t                    flags; // use helper functions below to read

    float                       minFloat;
    float                       maxFloat;
    uint32_t                    steps;
    int32_t                     minInt;
    int32_t                     maxInt;
    std::vector<StringEnumItem> stringEnum;
    std::vector<FloatEnumItem>  floatEnum;
    uint16_t                    maxStringLen;

public:
    // Make C wrapper functions friends so they can access private members
    friend uint32_t wing_node_definition_get_parent_id(node_definition_t);
    friend uint32_t wing_node_definition_get_id(node_definition_t);
    friend uint16_t wing_node_definition_get_index(node_definition_t);
    friend int wing_node_definition_get_name(node_definition_t, char*, size_t);
    friend int wing_node_definition_get_longname(node_definition_t, char*, size_t);
    friend float wing_node_definition_get_min_float(node_definition_t);
    friend float wing_node_definition_get_max_float(node_definition_t);
    friend uint32_t wing_node_definition_get_steps(node_definition_t);
    friend int32_t wing_node_definition_get_min_int(node_definition_t);
    friend int32_t wing_node_definition_get_max_int(node_definition_t);
    friend uint16_t wing_node_definition_get_max_string_len(node_definition_t);
    friend size_t wing_node_definition_get_string_enum_count(node_definition_t);
    friend int wing_node_definition_get_string_enum_item(node_definition_t, size_t, char*, size_t, char*, size_t);
    friend size_t wing_node_definition_get_float_enum_count(node_definition_t);
    friend int wing_node_definition_get_float_enum_item(node_definition_t, size_t, float*, char*, size_t);

    WingNode::Type          getType() const;
    WingNode::Unit          getUnit() const;
    bool                 isReadOnly() const;

    // convert between node names and ids
    static uint32_t    nodeNameToId(const std::string& fullname);
    static std::string nodeIdToName(uint32_t id);
};

class NodeData {
public:
    bool        hasString() const;
    std::string getString() const;
    bool        setString(const std::string& s);

    bool        hasFloat() const;
    float       getFloat() const;
    bool        setFloat(float f);

    bool          hasInt() const;
    int           getInt() const;
    bool          setInt(int i);

    void          clear();

private:
    int         _flags;
    std::string _s;
    int         _i;
    float       _f;
};

#endif // __WINGNODE_H
