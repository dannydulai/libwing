# Behringer Wing Digial Mixer Discovery Protocol

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

Note: when sending a packet using most operating system's networking APIs, you
will need to set the flag on the socket to allow sending to broadcast IPs.
