#include <cstring>

#include "WingNode.h"

WingNode::Type
NodeDefinition::getType() const
{
    return (WingNode::Type)((flags >> 4) & 0x0F);
}

WingNode::Unit
NodeDefinition::getUnit() const
{
    return (WingNode::Unit)(flags & 0x0F);
}

bool
NodeDefinition::isReadOnly() const
{
    return (flags >> 8) & 0x01;  // 9th bit
}

static std::map<uint32_t, std::string> _nodeHashToName;
static std::map<std::string, uint32_t> _nodeNameToHash = {
#include "wing-schema.cpp"
};

uint32_t
NodeDefinition::nodeNameToId(const std::string& fullname)
{
    return _nodeNameToHash[fullname];
}

std::string
NodeDefinition::nodeIdToName(uint32_t id)
{
    if (_nodeHashToName.size() != _nodeNameToHash.size()) {
        for (auto& [name, hash] : _nodeNameToHash) {
            _nodeHashToName[hash] = name;
        }
    }
    return _nodeHashToName[id];
}

std::string
NodeData::getString() const
{
    if (hasString()) return _s;
    if (hasInt()) return std::to_string(_i);
    if (hasFloat()) return std::to_string(_f);
    return "";
}
float
NodeData::getFloat() const
{
    if (hasFloat()) return _f;
    if (hasInt()) return _i;
    if (hasString()) return std::stof(_s);
    return 0;
}

int
NodeData::getInt() const
{
    if (hasInt()) return _i;
    if (hasFloat()) return _f;
    if (hasString()) return std::stoi(_s);
    return 0;
}

enum {
    NODE_DATA_TYPE_NONE    = 0,
    NODE_DATA_TYPE_STRING  = 1,
    NODE_DATA_TYPE_FLOAT   = 2,
    NODE_DATA_TYPE_INTEGER = 3,
};

bool
NodeData::setString(const std::string& s)
{
    bool ret = false;
    if (!hasString()) ret = true;
    else if (_s != s) ret = true;
    _flags = NODE_DATA_TYPE_STRING;
    _s = s;
    return ret;
}

bool
NodeData::setFloat(float f)
{
    bool ret = false;
    if (!hasFloat()) ret = true;
    else if (_f != f) ret = true;
    _flags = NODE_DATA_TYPE_FLOAT;
    _f = f;
    return ret;
}

bool
NodeData::setInt(int i)
{
    bool ret = false;
    if (!hasInt()) ret = true;
    else if (_i != i) ret = true;
    _flags = NODE_DATA_TYPE_INTEGER;
    _i = i;
    return ret;
}

bool
NodeData::hasString() const
{
    return _flags == NODE_DATA_TYPE_STRING;
}

bool
NodeData::hasFloat() const
{
    return _flags == NODE_DATA_TYPE_FLOAT;
}

bool
NodeData::hasInt() const
{
    return _flags == NODE_DATA_TYPE_INTEGER;
}

void
NodeData::clear()
{
    _flags = NODE_DATA_TYPE_NONE;
}
