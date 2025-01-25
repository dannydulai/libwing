#ifndef __WINGCONSOLE_H
#define __WINGCONSOLE_H

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
    void  read();
    void close();

    void setString(uint32_t id, const std::string& value) const;
    void setFloat (uint32_t id, float              value) const;
    void setInt   (uint32_t id, int                value) const;

    void requestNodeDefinition(uint32_t id) const;
    void       requestNodeData(uint32_t id) const;

    std::function<void(void)>               onRequestEnd;
    std::function<void(NodeDefinition)>     onNodeDefinition;
    std::function<void(uint32_t, NodeData)> onNodeData;

    static std::vector<DiscoveryInfo> discover(bool stopOnFirst = true);
    static WingConsole                 connect(const std::string &ip);
};

#endif // __WINGCONSOLE_H
