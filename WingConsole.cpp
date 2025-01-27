#include <chrono>
#include <thread>
#include <cstring>
#include <ctime>
#include <iomanip>
#include <iostream>
#include <map>
#include <sstream>
#include <stdexcept>
#include <vector>
#include <errno.h>
#include <fcntl.h>
#if _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#include <Ws2ipdef.h>
#define ISVALIDSOCKET(s) ((s) != INVALID_SOCKET)
#define GETSOCKETERRNO() (WSAGetLastError())
#else
#include <unistd.h>
#include <arpa/inet.h>
#include <sys/socket.h>
#define SOCKET int
#define ISVALIDSOCKET(s) ((s) >= 0)
#define GETSOCKETERRNO() (errno)
#define INVALID_SOCKET -1
#endif

#include "WingConsole.h"

using namespace std;
using namespace std::chrono;

class WingConsolePrivate {
public:
    SOCKET        _sock = INVALID_SOCKET;

    unsigned char _rx_buf[2048];
    size_t        _rx_buf_tail = 0; // Index for reading
    size_t        _rx_buf_size = 0; // Current number of bytes in buffer
    bool          _rx_esc = false;
    int           _rx_current_channel = -1;
    bool          _rx_hasinthepipe = false;
    unsigned char _rx_inthepipe = 0;

    unsigned char _getChar();
    void          _decode(int &channel, unsigned char &val);

    void close() {
        if (ISVALIDSOCKET(_sock)) {
            ::shutdown(_sock, 2);
            _sock = INVALID_SOCKET;
        }
    }
};

map<uint32_t, NodeData> _nodeData;
static time_point<system_clock> _keepAliveTime;

#define TIMEOUT_KEEP_ALIVE 7

vector<DiscoveryInfo>
WingConsole::scan(bool stopOnFirst)
{
    vector<DiscoveryInfo> discovered;

    SOCKET sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (!ISVALIDSOCKET(sock)) {
        throw std::system_error(GETSOCKETERRNO(), std::system_category(), "Error creating discovery socket");
    }

    int flags = fcntl(sock, F_GETFL, 0);
    fcntl(sock, F_SETFL, flags | O_NONBLOCK);

    int broadcastEnable = 1;
    if (setsockopt(sock, SOL_SOCKET, SO_BROADCAST, &broadcastEnable, sizeof(broadcastEnable)) < 0) {
        int err = GETSOCKETERRNO();
        ::shutdown(sock, 2);
        throw std::system_error(err, std::system_category(), "Error enabling broadcast sends on discovery socket");
    }

    struct sockaddr_in broadcastAddr;
    memset(&broadcastAddr, 0, sizeof(broadcastAddr));
    broadcastAddr.sin_family = AF_INET;
    broadcastAddr.sin_port = htons(2222);
    broadcastAddr.sin_addr.s_addr = inet_addr("255.255.255.255");

    char discoveryMsg[] = "WING?";
    if (sendto(sock, discoveryMsg, strlen(discoveryMsg), 0, (struct sockaddr*)&broadcastAddr, sizeof(broadcastAddr)) < 0) {
        int err = GETSOCKETERRNO();
        ::shutdown(sock, 2);
        throw std::system_error(err, std::system_category(), "Error sending broadcast discovery packet");
    }

    // Receive responses
    char buffer[1024];
    struct sockaddr_in senderAddr;
    socklen_t senderLen = sizeof(senderAddr);

    int i = 0;
    int got = 99999;
    while (i < 10) {
        auto received = ::recvfrom(sock, buffer, sizeof(buffer)-1, 0,
                                   (struct sockaddr*)&senderAddr, &senderLen);

        if (received > 0) {
            buffer[received] = '\0';
            string response(buffer);

            // Parse CSV response: WING,ip,name,model,serial,firmware
            stringstream ss(response);
            string token;
            vector<string> tokens;

            while (getline(ss, token, ',')) {
                tokens.push_back(token);
            }

            if (tokens.size() == 6 && tokens[0] == "WING") {
                DiscoveryInfo console;
                console.ip = tokens[1];
                console.name = tokens[2];
                console.model = tokens[3];
                console.serial = tokens[4];
                console.firmware = tokens[5];
                discovered.push_back(console);
                if (stopOnFirst)
                    break;
            }
            got = i;
            continue;

        } else if (received == 0) {
            break;

        } else {
            int err = GETSOCKETERRNO(); 
            if (err == EAGAIN || err == EWOULDBLOCK) {
                std::this_thread::sleep_for(std::chrono::milliseconds(500));
            } else {
                throw std::system_error(err, std::system_category(), "Error receiving discovery response");
                break;
            }
        }

        if (discovered.size() > 0 && i > got - 1) {
            break;
        }
        i++;
    }

    ::shutdown(sock, 2);
    return discovered;
}

void
WingConsole::close() {
    priv->close();
}

WingConsole
WingConsole::connect(const string &ip)
{
    WingConsole console;
    console.priv = new WingConsolePrivate();
    console.priv->_sock = socket(AF_INET, SOCK_STREAM, 0);
    if (console.priv->_sock < 0) {
        throw std::system_error(GETSOCKETERRNO(), std::system_category(), "Failed to create socket");
    }

#if _WIN32
    DWORD timeout = TIMEOUT_KEEP_ALIVE * 1000;
    setsockopt(console._sock, SOL_SOCKET, SO_RCVTIMEO, (const char*)&timeout, sizeof timeout);
#else
    struct timeval tv;
    tv.tv_sec = TIMEOUT_KEEP_ALIVE;
    tv.tv_usec = 0;
    setsockopt(console.priv->_sock, SOL_SOCKET, SO_RCVTIMEO, (const char*)&tv, sizeof tv);
#endif

    struct sockaddr_in serverAddr;
    memset(&serverAddr, 0, sizeof(serverAddr));
    serverAddr.sin_family = AF_INET;
    serverAddr.sin_port = htons(2222);  // console WING console port
    serverAddr.sin_addr.s_addr = inet_addr(ip.c_str());


    if (::connect(console.priv->_sock, (struct sockaddr*)&serverAddr, sizeof(serverAddr)) < 0) {
        int err = GETSOCKETERRNO();
        throw std::system_error(err, std::system_category(), "Failed to connect to console");
    }

    _keepAliveTime = system_clock::now();

    unsigned char buf[] = { 0xdf, 0xd1 }; // switch to channel 2 (Audio Ending & Control requests)
    if (::send(console.priv->_sock, buf, sizeof(buf), 0) != sizeof(buf)) {
        throw std::system_error(GETSOCKETERRNO(), std::system_category(), "Failed to send message");
    }

    return console;
}

static int
formatId(int id, unsigned char *buf, unsigned char prefix, unsigned char suffix) {
    auto b = buf;
    unsigned char c;
    *buf++ = prefix;
    c = (unsigned char)( id >> 24        ); if (c == 0xdf) { *buf++ = 0xdf; *buf++ = 0xde; } else *buf++ = c;
    c = (unsigned char)((id >> 16) & 0xff); if (c == 0xdf) { *buf++ = 0xdf; *buf++ = 0xde; } else *buf++ = c;
    c = (unsigned char)((id >> 8 ) & 0xff); if (c == 0xdf) { *buf++ = 0xdf; *buf++ = 0xde; } else *buf++ = c;
    c = (unsigned char)((id      ) & 0xff); if (c == 0xdf) { *buf++ = 0xdf; *buf++ = 0xde; } else *buf++ = c;
    *buf++ = suffix;
    return (int)(buf - b);
}

static void
keepAlive(int sock)
{
    // Some computation here
    auto now = system_clock::now();

    duration<double> elapsed_seconds = now-_keepAliveTime;
    if (elapsed_seconds.count() > TIMEOUT_KEEP_ALIVE) {
        unsigned char buf[] = { 0xdf, 0xd1 }; // switch to channel 2 (Audio Ending & Control requests)
        if (::send(sock, buf, sizeof(buf), 0) != sizeof(buf)) {
            throw std::system_error(GETSOCKETERRNO(), std::system_category(), "Failed to send keepalive message");
        }
        _keepAliveTime = system_clock::now();
    }
}

unsigned char
WingConsolePrivate::_getChar()
{
    keepAlive(_sock);
    // If buffer is empty, read more data from socket
    if (_rx_buf_size == 0) {
        while (true) {
            auto n = ::recv(_sock, (char*)_rx_buf, sizeof(_rx_buf), 0);
            if (n < 0) {
                int err = GETSOCKETERRNO();
                if (err == EAGAIN || err == EWOULDBLOCK) {
                    keepAlive(_sock);
                    continue;
                } else {
                    throw std::system_error(err, std::system_category(), "Error reading from socket");
                }
            } else if (n == 0) {
                throw std::system_error(ECONNRESET, std::system_category(), "Connection closed");
            }
            _rx_buf_tail = 0;
            _rx_buf_size = n;
            break;
        }
    }

    _rx_buf_size--;
    return _rx_buf[_rx_buf_tail++];
}

#define NRP_ESCAPE_CODE 0xdf
#define NRP_CHANNEL_ID_BASE 0xd0
#define NRP_NUM_CHANNELS 14

void
WingConsolePrivate::_decode(int &channel, unsigned char &val)
{
    if (_rx_hasinthepipe) {
        channel = _rx_current_channel;
        val = _rx_inthepipe;
        _rx_hasinthepipe = false;
        return;
    }

    while (true) {
        unsigned char c = _getChar();

        if (c == NRP_ESCAPE_CODE && !_rx_esc) {
            _rx_esc = true;
        } else {
            if (_rx_esc) {
                if (c != NRP_ESCAPE_CODE) {
                    _rx_esc = false;
                    if (c == NRP_ESCAPE_CODE - 1) {
                        c = NRP_ESCAPE_CODE;
                    } else if (c >= NRP_CHANNEL_ID_BASE && c < NRP_CHANNEL_ID_BASE + NRP_NUM_CHANNELS) {
                        _rx_current_channel = c - NRP_CHANNEL_ID_BASE;
                        continue;
                    } else if (_rx_current_channel >= 0) {
                        channel = _rx_current_channel;
                        val = NRP_ESCAPE_CODE;
                        _rx_hasinthepipe = true;
                        _rx_inthepipe = c;
                        return;
                    }
                }
            }
            if (_rx_current_channel >= 0) {
                channel = _rx_current_channel;
                val = c;
                return;
            }
        }
    }
}

#define read8(x)  do { unsigned char c; priv->_decode(channel, c); x = c; } while(0)
#define read16(x) do { unsigned char c; priv->_decode(channel, c); x = c << 8; priv->_decode(channel, c); x |= c; } while(0)
#define read32(x) do { unsigned char c; priv->_decode(channel, c); x = c << 24; priv->_decode(channel, c); x |= c << 16; priv->_decode(channel, c); x |= c << 8; priv->_decode(channel, c); x |= c; } while(0)
#define readfloat(x) do { unsigned char c; priv->_decode(channel, c); uint32_t tmp = c << 24; priv->_decode(channel, c); tmp |= c << 16; priv->_decode(channel, c); tmp |= c << 8; priv->_decode(channel, c); tmp |= c; x = *(float*)&tmp;} while(0)

void
WingConsole::read()
{
    try {
        int currentNode = -1;

        while (true) {
            int tmp, channel;
            unsigned char cmd;
            read8(cmd);

            if (cmd == 0x00) {
                if (_nodeData[currentNode].setInt(0)) {
                    if (onNodeData) onNodeData(currentNode, _nodeData[currentNode]);
                }

            } else if (cmd == 0x01) {
                if (_nodeData[currentNode].setInt(1)) {
                    if (onNodeData) onNodeData(currentNode, _nodeData[currentNode]);
                }

            } else if (cmd >= 0x02 && cmd <= 0x3f) {
                if (_nodeData[currentNode].setInt((int)cmd)) {
                    if (onNodeData) onNodeData(currentNode, _nodeData[currentNode]);
                }

            } else if (cmd >= 0x40 && cmd <= 0x7f) {
                cerr << "REQUEST: FAST NODE INDEX:" << (int)(cmd-0x40+1) << endl;

            } else if (cmd >= 0x80 && cmd <= 0xbf) {
                int len = (int)(cmd-0x80+1);
                string s;
                for (int j = 0; j < len; j++) {
                    unsigned char ch;
                    read8(ch);
                    s += ch;
                }
                if (_nodeData[currentNode].setString(s)) {
                    if (onNodeData) onNodeData(currentNode, _nodeData[currentNode]);
                }

            } else if (cmd >= 0xc0 && cmd <= 0xcf) {
                int len = (int)(cmd-0xc0+1);
                cerr << "REQUEST: FAST NODE NAME:" << (int)(cmd-0xc0+1) << endl;

            } else if (cmd == 0xd0) {
                if (_nodeData[currentNode].setString("")) {
                    if (onNodeData) onNodeData(currentNode, _nodeData[currentNode]);
                }

            } else if (cmd == 0xd1) {
                int len;
                read8(len);
                len++;
                string s;
                for (int j = 0; j < len; j++) {
                    unsigned char ch;
                    read8(ch);
                    s += ch;
                }
                if (_nodeData[currentNode].setString(s)) {
                    if (onNodeData) onNodeData(currentNode, _nodeData[currentNode]);
                }

            } else if (cmd == 0xd2) {
                read16(tmp);
                tmp++;
                cerr << "REQUEST: NODE INDEX: " << tmp << endl;

            } else if (cmd == 0xd3) {
                read16(tmp);
                if (_nodeData[currentNode].setInt(tmp)) {
                    if (onNodeData) onNodeData(currentNode, _nodeData[currentNode]);
                }

            } else if (cmd == 0xd4) {
                read32(tmp);
                if (_nodeData[currentNode].setInt(tmp)) {
                    if (onNodeData) onNodeData(currentNode, _nodeData[currentNode]);
                }

            } else if (cmd == 0xd5) {
                float f;
                readfloat(f);
                if (_nodeData[currentNode].setFloat(f)) {
                    if (onNodeData) onNodeData(currentNode, _nodeData[currentNode]);
                }

            } else if (cmd == 0xd6) {
                float f;
                readfloat(f);
                if (_nodeData[currentNode].setFloat(f)) {
                    if (onNodeData) onNodeData(currentNode, _nodeData[currentNode]);
                }

            } else if (cmd == 0xd7) {
                read32(currentNode);

            } else if (cmd == 0xd8) {
                cerr << "??????? CLICK" << endl;

            } else if (cmd == 0xd9) {
                char step;
                read8(step);
                cerr << "??????? STEP: " << (int)step << endl;

            } else if (cmd == 0xda) {
                cerr << "REQUEST: TREE: GOTO ROOT" << endl;

            } else if (cmd == 0xdb) {
                cerr << "REQUEST: TREE: GO UP 1" << endl;

            } else if (cmd == 0xdc) {
                cerr << "REQUEST: DATA" << endl;

            } else if (cmd == 0xdd) {
                cerr << "REQUEST: CURRENT NODE DEFINITION" << endl;

            } else if (cmd == 0xde) {
                if (onRequestEnd) onRequestEnd();

            } else if (cmd == 0xdf) { // node definition response
                NodeDefinition node;
                int len;
                read16(len);
                if (len == 0) {
                    read32(len);
                }
                read32(node.parentId);
                read32(node.id);
                read16(node.index);
                read8(len);
                for (int j = 0; j < len; j++) {
                    unsigned char ch;
                    read8(ch);
                    node.name += ch;
                }
                read8(len);
                for (int j = 0; j < len; j++) {
                    unsigned char ch;
                    read8(ch);
                    node.longName += ch;
                }

                read16(node.flags);

                if (node.getType() == WingNode::TYPE_STRING) {
                    read16(node.maxStringLen);
                } else if (node.getType() == WingNode::TYPE_LINEAR_FLOAT || node.getType() == WingNode::TYPE_LOGARITHMIC_FLOAT) {
                    readfloat(node.minFloat);
                    readfloat(node.maxFloat);
                    read32(node.steps);
                } else if (node.getType() == WingNode::TYPE_INTEGER) {
                    read32(node.minInt);
                    read32(node.maxInt);
                } else if (node.getType() == WingNode::TYPE_STRING_ENUM) {
                    int num;
                    read16(num);
                    for (int k = 0; k < num; k++) {
                        StringEnumItem item;
                        int len;
                        read8(len);
                        for (int j = 0; j < len; j++) {
                            unsigned char ch;
                            read8(ch);
                            item.item += ch;
                        }
                        read8(len);
                        for (int j = 0; j < len; j++) {
                            unsigned char ch;
                            read8(ch);
                            item.longitem += ch;
                        }
                        node.stringEnum.push_back(item);
                    }
                } else if (node.getType() == WingNode::TYPE_FLOAT_ENUM) {
                    int num;
                    read16(num);
                    for (int k = 0; k < num; k++) {
                        FloatEnumItem item;
                        readfloat(item.item);
                        int len;
                        read8(len);
                        for (int j = 0; j < len; j++) {
                            unsigned char ch;
                            read8(ch);
                            item.longitem += ch;
                        }
                        node.floatEnum.push_back(item);
                    }
                }

                if (onNodeDefinition) onNodeDefinition(node);
            } else {
                cerr << "Received UNKNOWN BYTE: " << hex << setw(2) << setfill('0') << (int)cmd << endl;
            }
        }
    } catch (std::system_error &e) {
        if (e.code().value() == ECONNRESET) {
            return;
        }
        throw;
    }
}

void
WingConsole::requestNodeDefinition(uint32_t id) const
{
    unsigned char buf[16];
    int len;
    if (id == 0) {
        buf[0] = 0xda;
        buf[1] = 0xdd;
        len = 2;
    } else {
        len = formatId(id, buf, 0xd7, 0xdd);
    }
    if (::send(priv->_sock, buf, len, 0) != len) {
        throw std::system_error(GETSOCKETERRNO(), std::system_category(), "Failed to send get-node-definition message");
    }
}

void
WingConsole::requestNodeData(uint32_t id) const
{
    unsigned char buf[16];
    int len;
    if (id == 0) {
        buf[0] = 0xda;
        buf[1] = 0xdc;
        len = 2;
    } else {
        len = formatId(id, buf, 0xd7, 0xdc);
    }
    if (::send(priv->_sock, buf, len, 0) != len) {
        throw std::system_error(GETSOCKETERRNO(), std::system_category(), "Failed to send get-node-data message");
    }
}

void
WingConsole::setString(uint32_t id, const string& value) const
{
    unsigned char buf[272];
    int len = formatId(id, buf, 0xd7, 0x0); // this suffix is wrong, we will clobber it below
    len--;
    if (value.size() == 0) {
        buf[len++] = 0xd0;
    } else if (value.size() <= 64) {
        buf[len++] = 0x3F + value.size();
    } else if (value.size() <= 256) {
        buf[len++] = 0xd1;
        buf[len++] = value.size() - 1;
    } else {
        throw runtime_error("String too long");
    }
    for (char c : value) {
        buf[len++] = c;
    }
    if (::send(priv->_sock, buf, len, 0) != len) {
        throw std::system_error(GETSOCKETERRNO(), std::system_category(), "Failed to send set-node-int message");
    }
}

void
WingConsole::setFloat(uint32_t id, float value) const
{
    unsigned char buf[16];
    int len = formatId(id, buf, 0xd7, 0xd5);
    uint32_t v = *(uint32_t*)&value;
    buf[len++] = (v >> 24) & 0xff;
    buf[len++] = (v >> 16) & 0xff;
    buf[len++] = (v >> 8) & 0xff;
    buf[len++] = v & 0xff;

    if (::send(priv->_sock, buf, len, 0) != len) {
        throw std::system_error(GETSOCKETERRNO(), std::system_category(), "Failed to send set-node-int message");
    }
}

void
WingConsole::setInt(uint32_t id, int32_t value) const
{
    unsigned char buf[16];
    int len = formatId(id, buf, 0xd7, 0x0); // this suffix is wrong, we will clobber it below
    len--;
    if (value >= 0 && value <= 0x3f) {
        buf[len++] = value;
    } else if (value <= 0xffff) {
        buf[len++] = 0xd3;
        buf[len++] = value >> 8;
        buf[len++] = value & 0xff;
    } else {
        buf[len++] = 0xd4;
        buf[len++] = (value >> 24) & 0xff;
        buf[len++] = (value >> 16) & 0xff;
        buf[len++] = (value >> 8) & 0xff;
        buf[len++] = value & 0xff;
    }
    if (::send(priv->_sock, buf, len, 0) != len) {
        throw std::system_error(GETSOCKETERRNO(), std::system_category(), "Failed to send set-node-int message");
    }
}
