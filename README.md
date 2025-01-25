# Introduction

This software includes a C/C++ library for discovering and controlling
the [Behringer Wing](https://www.behringer.com/behringer/wing) mixer over the
local network. It also includes a series of utilities built using this library.

![Behringer Wing Family](wing.jpg)

# Usage

## The library

This library is called **libwing**. It is built as a static
library, but not too hard to change if needed (just add it to the
CMakeLists.txt).

#### Dependencies

- MacOS, Linux, Windows
- cmake
- make
- C++20 compiler

#### Building

To build the software, do the following:

```
cmake .
make
```

This will build the **libwing** library and a couple of utilities that use it.
Output binaries will end up in the build/ subdirectory.

#### Using the library

You can find informatiron on using the library in the [SDK docuemntation](SDK.md).

## The utilities

### wingschema

**wingschema** can be run with zero arguments. It will discover the Wing on the
network automatically. Once found, it will request every node and parameter's
schema and save them to two files. This process takes about a minute over the
network.

`wing-schema.jsonl` is for you to read, learn, and use as you want. It is a
great resource for understanding the Wing's capabilities.

`wing-schema.cpp` is used by **libwing**. It contains a "name to ID" mapping of
all the Wing's nodes and parameters. Just re-building **libwing** picks up this
new file. **libwing** uses this mapping to provide a nice way to convert
between names and the Wing node/parameter IDs. I include a wing-schema.cpp in
the repo so you don't have to run this, but you can always run it again if you
want to get the latest schema from newer versions of the firmware.

### wingmon

**wingmon** is a utility that connects to the Wing and prints out
property changes. Just run it with no arguments (it'll discover the Wing on the
network for you) and go manipulate your Wing via the console or one of the Wing
apps. You'll see all the things that changed printed to the console.



# Protocols

There are 3 relevant protocols for controlling the Wing:

- The Discovery protocol
- The OSC protocol
- The Native protocol

The OSC protocol and Native protocol are documented
[here](https://cdn.mediavalet.com/aunsw/musictribe/mzolJdOzu0WZG59pX2LDkA/drJQVBUjakq76Xn2GcaT0Q/Original/WING%20Remote%20Protocols%20v3.0.5.pdf)
(as of Wing firmware v3.0.5) by Patrick-Gilles Maillot.

A copy of that document is checked in to this repo just in case the above link
is not available. The file is called Wing-Remote-Protocols.pdf.

## Discovery Protocol

This simple protocol allows a client on the network to discover Wing devices on
the network. **libwing** implements this discovery protocol.

To discover a Wing device on your network, send a 5 byte UDP packet containing
the following bytes to your network broadcast IP on UDP port 2222.

```
0x57, 0x49, 0x4E, 0x47, 0x3F       // ASCII for W, I, N, G, ?
```

Your Wing unit(s) will respond with a UDP packet containing a commma-separated
list of fields. The fields are as follows:

```
WING,ip,name,model,serial,firmware
```

For example, my Wing Compact running firmware 3.0.5 looks like this (I've
altered the identifying information for privacy reasons):

```
WING,192.168.1.19,WING-PP-93892216,wing-compact,01XXXUX06X3AEX,3.0.5-0-g0c2b9d4a:release
```

Other models seem to be:

- "wing" (possibly the full Wing in white)
- "wing-rack" (the rackmount version)
- "wing-bk" (a black version of the full Wing console)
- "ngc-full" (the full Wing console in white)

I can only confirm the "wing-compact" model, as that is the only one I have
access to. If someone would like to report the model for any others, please do
so with a pull request updating this doc.

##### Broadcast IP

You can compute the broadcast IP by taking the bitwise OR of the IP and the
bitwise NOT of the subnet mask (broadcast = ip | ~subnet). For example, if you
are 192.168.1.23 / 255.255.255.0, then your broadcast would be 192.168.1.255.

You can also send it to 255.255.255.255 as your router will most likely block
your packet from leaving your local network.

## OSC (Open Sound Control) Protocol

[OSC](https://en.wikipedia.org/wiki/Open_Sound_Control) is a standard protocol
for communication with mixers, synthesizers, and other audio equipment. It is a
text-based protocol that is easy to understand and implement. Unfortunately, it
comes with some limitations and overhead that make it less performant than the
Native protocol.

**libwing** does not implement the OSC protocol, but there are many other
libraries available that do support OSC.

## Native protocol

The Wing protocol is a binary protocol that is more far efficient than OSC and
operates over TCP. It combines the query/response as well as subscriptions on
one TCP connection. It is used by all the Behringer Wing apps to communicate
with the Wing.

**libwing** implements the Native protocol.

