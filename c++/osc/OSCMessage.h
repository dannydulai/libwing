#ifndef OSCMESSAGE_H
#define OSCMESSAGE_H

#include <iostream>
#include <cstring>
#include <sys/socket.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <string>
#include <vector>
#include <variant>

class OSCMessage {
public:
    std::string address;
    std::vector<std::variant<float,int32_t,std::string>> values;

    friend std::ostream & operator<<(std::ostream &stream, const OSCMessage &object);

    OSCMessage();
    OSCMessage(const std::string& address);
    static OSCMessage fromBuffer(const char *buffer, size_t buflen);

    std::vector<char> toBuffer();
};

#endif
