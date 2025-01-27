#ifndef __WINGCONSOLE_H
#define __WINGCONSOLE_H

#include <string>
#include <vector>
#include <functional>

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
public:
    // read() will process messages from the Wing device and block until the
    // connection is closed
    void read(); // be sure to set callbacks below before calling read()

    // set these before calling read() to get data (if you are requesting it)
    std::function<void(void)> onRequestEnd;
    std::function<void(NodeDefinition)> onNodeDefinition;
    std::function<void(uint32_t, NodeData)> onNodeData;

    // set the value of a node
    void setString(uint32_t id, const std::string &value) const;
    void setFloat(uint32_t id, float value) const;
    void setInt(uint32_t id, int value) const;

    // request information from the Wing device
    void requestNodeDefinition(uint32_t id) const;
    void requestNodeData(uint32_t id) const;

    // scan to discover a Wing device
    static std::vector<DiscoveryInfo> scan(bool stopOnFirst = true);

    // connect to a wing device
    static WingConsole connect(const std::string &ip);

    // close the connection to the Wing device
    void close();

private:
    WingConsolePrivate *priv;
};

#endif // __WINGCONSOLE_H
