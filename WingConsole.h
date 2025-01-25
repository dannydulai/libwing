#ifndef WINGNODE_H
#define WINGNODE_H

#include <string>
#include <vector>

#include "WingNode.h"

struct DiscoveryInfo {
    std::string ip;
    std::string name;
    std::string model;
    std::string serial;
    std::string firmware;
};

class WingConsolePrivate;
class WingConsole {
    WingConsolePrivate *priv;
public:
    void read();
    void close();

    void setString(uint32_t id, const std::string& value);
    void setFloat (uint32_t id, float value);
    void setInt   (uint32_t id, int value);

    void requestNodeDefinition(uint32_t id);
    void       requestNodeData(uint32_t id);

    std::function<void(void)>               onRequestEnd;
    std::function<void(NodeDefinition)>     onNodeDefinition;
    std::function<void(uint32_t, NodeData)> onNodeData;

    static std::vector<DiscoveryInfo> discover(bool stopOnFirst = true);
    static WingConsole                 connect(const std::string &ip);
};

#endif
