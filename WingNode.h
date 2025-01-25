#ifndef __WINGNODE_H
#define __WINGNODE_H
#include <string>
#include <map>
#include <vector>

enum NodeUnit {
    NODE_UNIT_NONE         = 0,
    NODE_UNIT_DB           = 1,
    NODE_UNIT_PERCENT      = 2,
    NODE_UNIT_MILLISECONDS = 3,
    NODE_UNIT_HERTZ        = 4,
    NODE_UNIT_METERS       = 5,
    NODE_UNIT_SECONDS      = 6,
    NODE_UNIT_OCTAVES      = 7,
};
enum NodeType {
    NODE_TYPE_NODE              = 0,
    NODE_TYPE_LINEAR_FLOAT      = 1,
    NODE_TYPE_LOGARITHMIC_FLOAT = 2,
    NODE_TYPE_FADER_LEVEL       = 3,
    NODE_TYPE_INTEGER           = 4,
    NODE_TYPE_STRING_ENUM       = 5,
    NODE_TYPE_FLOAT_ENUM        = 6,
    NODE_TYPE_STRING            = 7
};

struct StringEnumItem {
    std::string item;
    std::string longitem;
};

struct FloatEnumItem {
    float       item;
    std::string longitem;
};

struct NodeDefinition {
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

    NodeType                getType() const;
    NodeUnit                getUnit() const;
    bool                 isReadOnly() const;

    // convert between node names and ids
    static uint32_t    nodeNameToId(const std::string& fullname);
    static std::string nodeIdToName(uint32_t id);
};

class NodeData {
    int         _flags;
    std::string _s;
    int         _i;
    float       _f;

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
};

#endif // __WINGNODE_H
