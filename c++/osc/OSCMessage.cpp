#include <stdexcept>
#include <cctype>
#include <iomanip>
#include <iostream>
#include <cstring>
#include <sys/socket.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <string>
#include <vector>
#include <fcntl.h>
#include <variant>

#include "OSCMessage.h"

static void printBuffer(const char* buffer, size_t len) {
    for (size_t i = 0; i < len; ++i) {
        char c = buffer[i];
        if (std::isprint(static_cast<unsigned char>(c))) {
            std::cout << c;
        } else {
            std::cout << "{" << std::hex << std::uppercase << std::setw(2) << std::setfill('0')
                      << static_cast<int>(static_cast<unsigned char>(c)) << "}";
        }
    }
    std::cout << std::endl;
}

std::ostream&
operator<<(std::ostream &stream, const OSCMessage &object)
{
    stream << "OSCMessage(" << object.address << " = [";

    int i = 0;
    while (i < object.values.size()) {
        if (i > 0) {
            stream << ", ";
        }
        if (const int* pval = std::get_if<int>(&object.values[i])) {
            stream << *pval;
        } else if (const float* pval = std::get_if<float>(&object.values[i])) {
            stream << *pval;
        } else if (const std::string* pval = std::get_if<std::string>(&object.values[i])) {
            stream << *pval;
        }
        i++;
    }
    stream << "])";
    return stream;
}

OSCMessage::OSCMessage()
{
}

OSCMessage::OSCMessage(const std::string& address) {
    this->address = address;
}

static std::string
_decodeString(const char* data, size_t& offset, size_t buflen)
{
    const char* end = std::find(data + offset, data + buflen, '\0');
    std::string s(data+offset, end);
    offset += ((s.length() + 4) & ~3); // Move to next 4-byte boundary
    return s;
}

OSCMessage
OSCMessage::fromBuffer(const char *buffer, size_t buflen)
{
    if (buflen < 4) throw std::length_error("Invalid OSC message buffer (length < 4)");

    size_t offset = 0;
    OSCMessage m(_decodeString(buffer, offset, buflen));

    if (offset >= buflen) throw std::length_error("Invalid OSC message buffer (length < 8)");

    std::string typeTag = _decodeString(buffer, offset, buflen);
    if (typeTag[0] != ',') {
        throw std::runtime_error(std::format("Invalid OSC message buffer (type tag doesn't start with comma): >>>{}<<< >>>{}<<<", buffer, typeTag));
    }

    for (int i = 1; i < typeTag.length(); i++) {
        if (offset >= buflen) break;

        switch (typeTag[i]) {
            case 'f':
                if (offset + 4 <= buflen) {
                    union {
                        uint32_t i;
                        float f;
                    } converter;
                    converter.i = ntohl(*reinterpret_cast<const uint32_t*>(buffer + offset));
                    offset += 4;
                    m.values.push_back(converter.f);
                }
                break;
            case 'i':
                if (offset + 4 <= buflen) {
                    int32_t value = ntohl(*reinterpret_cast<const uint32_t*>(buffer + offset));
                    offset += 4;
                    m.values.push_back(value);
                }
                break;
            case 's':
                if (offset < buflen) {
                    std::string value = _decodeString(buffer, offset, buflen);
                    m.values.push_back(value);
                }
                break;
        }
    }
    return m;
}

std::vector<char>
OSCMessage::toBuffer()
{
    std::vector<char> packet;

    // Add OSC address
    packet.insert(packet.end(), address.begin(), address.end());

    // Pad to multiple of 4 bytes
    while (packet.size() % 4 != 0) {
        packet.push_back('\0');
    }

    if (values.empty()) return packet;

    // Add type tag
    packet.push_back(',');
    for (int i = 0; i < values.size(); i++) {
        if (const int* pval = std::get_if<int>(&values[i])) {
            packet.push_back('i');
        } else if (const float* pval = std::get_if<float>(&values[i])) {
            packet.push_back('f');
        } else if (const std::string* pval = std::get_if<std::string>(&values[i])) {
            packet.push_back('s');
        }
    }
    packet.push_back(0);

    // Pad to multiple of 4 bytes
    while (packet.size() % 4 != 0) {
        packet.push_back('\0');
    }

    for (int i = 0; i < values.size(); i++) {
        if (const int* pval = std::get_if<int>(&values[i])) {
            packet.push_back((*pval >> 24) & 0xff);
            packet.push_back((*pval >> 16) & 0xff);
            packet.push_back((*pval >>  8) & 0xff);
            packet.push_back((*pval      ) & 0xff);
        } else if (const float* pval = std::get_if<float>(&values[i])) {
            // Add float value in big-endian format
            union {
                float f;
                uint32_t i;
            } converter;
            converter.f = *pval;
            uint32_t be = htonl(converter.i);
            packet.push_back((be >> 24) & 0xff);
            packet.push_back((be >> 16) & 0xff);
            packet.push_back((be >>  8) & 0xff);
            packet.push_back((be      ) & 0xff);
        } else if (const std::string* pval = std::get_if<std::string>(&values[i])) {
            packet.insert(packet.end(), pval->begin(), pval->end());
            packet.push_back(0);

            // Pad to multiple of 4 bytes
            while (packet.size() % 4 != 0) {
                packet.push_back('\0');
            }
        }
    }

    return packet;
}
