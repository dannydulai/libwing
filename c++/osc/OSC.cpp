#include <cstring>
#include <sys/socket.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <vector>
#include <fcntl.h>
#include <errno.h>

#include "WingConsole.h"

static void
_sendTo(int sock, struct sockaddr_in serverAddr,  const std::vector<char>& packet)
{
    ssize_t sent = sendto(sock, packet.data(), packet.size(), 0, (struct sockaddr*)&serverAddr, sizeof(serverAddr));
    if (sent < 0) throw std::runtime_error("Failed to send packet");
}

void
OSC::close()
{
    shutdown(_sock, SHUT_RDWR);
}

OSCMessage
OSC::recv()
{
    char buf[32768];
    struct sockaddr_in senderAddr;
    socklen_t senderLen = sizeof(senderAddr);

    while (true) {
        ssize_t buflen = recvfrom(_sock, buf, sizeof(buf), 0, (struct sockaddr*)&senderAddr, &senderLen);

        if (buflen > 0) {
            return OSCMessage::fromBuffer(buf, buflen);
        } else if (buflen < 0 && errno != EAGAIN && errno != EWOULDBLOCK) {
            throw std::runtime_error("Error receiving message");
        }
        usleep(1000);
    }
}

std::string
OSC::getString(const std::string &address)
{
    return std::get<std::string>(get(address).values[0]);
}

float
OSC::getFloat(const std::string &address)
{
    return std::get<float>(get(address).values[0]);
}

int
OSC::getInt(const std::string &address)
{
    return std::get<int>(get(address).values[0]);
}

void
OSC::setString(const std::string &address, const std::string &value)
{
    OSCMessage msg(address);
    msg.values.push_back(value);
    set(msg);
}

void
OSC::setFloat(const std::string &address, float value) {
    OSCMessage msg(address);
    msg.values.push_back(value);
    set(msg);
}

void
OSC::setInt(const std::string &address, int value) {
    OSCMessage msg(address);
    msg.values.push_back(value);
    set(msg);
}

void
OSC::set(OSCMessage msg)                                      
{
    _sendTo(_sock, _serverAddr, msg.toBuffer());
}

OSCMessage
OSC::get(const std::string& address)
{
    OSCMessage message(address);
    _sendTo(_sock, _serverAddr, message.toBuffer());
    return recv();
}

Subscription
OSC::subscribe()
{
    Subscription sub;
    sub.sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (sub.sock < 0)
        throw std::runtime_error("Failed to create subscription socket");

    int flags = fcntl(sub.sock, F_GETFL, 0);
    fcntl(sub.sock, F_SETFL, flags | O_NONBLOCK);

    // XXX       

    OSCMessage message("/*S");
    _sendTo(_sock, _serverAddr, message.toBuffer());
    return sub;
}

