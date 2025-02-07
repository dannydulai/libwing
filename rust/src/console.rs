use std::net::{TcpStream, UdpSocket};
use std::io::{Read, Write};
use std::time::Duration;

use crate::{Result, Error};
use crate::node::{NodeDefinition, NodeData, NodeType, NodeUnit, StringEnumItem, FloatEnumItem};
use crate::Response;

const RX_BUFFER_SIZE: usize = 2048;

pub struct DiscoveryInfo {
    pub ip:       String,
    pub name:     String,
    pub model:    String,
    pub serial:   String,
    pub firmware: String,
}

pub struct WingConsole {
    stream:             TcpStream,
    current_node_id:    i32,
    keep_alive_timer:   std::time::Instant,
    rx_buf:             [u8; RX_BUFFER_SIZE],
    rx_buf_tail:        usize,
    rx_buf_size:        usize,
    rx_esc:             bool,
    rx_current_channel: i8,
    rx_has_in_pipe:     Option<u8>,
}

impl WingConsole {
    pub fn scan(stop_on_first: bool) -> Result<Vec<DiscoveryInfo>> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_broadcast(true)?;
        socket.set_read_timeout(Some(Duration::from_millis(500))).unwrap();

        let mut results = Vec::new();
        let mut attempts = 0;

        socket.send_to(b"WING?", "255.255.255.255:2222")?;
        while attempts < 10 {
            let mut buf = [0u8; 1024];
            match socket.recv_from(&mut buf) {
                Ok((received, _)) => {
                    if let Ok(response) = String::from_utf8(buf[..received].to_vec()) {
                        let tokens: Vec<&str> = response.split(',').collect();
                        if tokens.len() >= 6 && tokens[0] == "WING" {
                            results.push(DiscoveryInfo {
                                ip:       tokens[1].to_string(),
                                name:     tokens[2].to_string(),
                                model:    tokens[3].to_string(),
                                serial:   tokens[4].to_string(),
                                firmware: tokens[5].to_string(),
                            });
                            if stop_on_first {
                                break;
                            }
                        }
                    }
                }
                Err(_) => {
                    attempts += 1;
                }
            }
        }

        Ok(results)
    }

    pub fn connect(ip: &str) -> Result<Self> {
        let mut stream = TcpStream::connect((ip, 2222))?;
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;
        stream.write_all(&[0xdf, 0xd1])?;
        
        Ok(Self {
            stream,
            rx_buf: [0; RX_BUFFER_SIZE],
            rx_buf_tail: 0,
            rx_buf_size: 0,
            rx_esc: false,
            rx_current_channel: -1,
            rx_has_in_pipe: None,
            current_node_id: 0,
            keep_alive_timer: std::time::Instant::now(),
        })
    }

    pub fn read(&mut self) -> Result<Response> {
        loop {
            let (ch, cmd) = self.decode_next()?;
            //println!("Channel: {}, Command: {:X}", ch, cmd);
            if cmd <= 0x3f {
                let v = cmd as i32;
                return Ok(Response::NodeData(self.current_node_id, NodeData::with_i32(v)));
            } else if cmd <= 0x7f {
                let v = cmd - 0x40 + 1;
                println!("REQUEST: NODE INDEX: {}", v);
            } else if cmd <= 0xbf {
                let len = cmd - 0x80 + 1;
                let v = self.read_string(ch, len as usize)?;
                return Ok(Response::NodeData(self.current_node_id, NodeData::with_string(v)));
            } else if cmd <= 0xcf {
                let len = cmd - 0xc0 + 1;
                let v = self.read_string(ch, len as usize)?;
                return Ok(Response::NodeData(self.current_node_id, NodeData::with_string(v)));
            } else if cmd == 0xd0 {
                let v = String::new();
                return Ok(Response::NodeData(self.current_node_id, NodeData::with_string(v)));
            } else if cmd == 0xd1 {
                let len = self.read_u8(ch)? + 1;
                let v = self.read_string(ch, len as usize)?;
                return Ok(Response::NodeData(self.current_node_id, NodeData::with_string(v)));
            } else if cmd == 0xd2 {
                let v = self.read_u16(ch)? + 1;
                println!("REQUEST: NODE INDEX: {}", v);
            } else if cmd == 0xd3 {
                let v = self.read_i16(ch)?;
                return Ok(Response::NodeData(self.current_node_id, NodeData::with_i16(v)));
            } else if cmd == 0xd4 {
                let v = self.read_i32(ch)?;
                return Ok(Response::NodeData(self.current_node_id, NodeData::with_i32(v)));
            } else if cmd == 0xd5 || cmd == 0xd6 {
                let v = self.read_f(ch)?;
                return Ok(Response::NodeData(self.current_node_id, NodeData::with_float(v)));
            } else if cmd == 0xd7 {
                self.current_node_id = self.read_i32(ch)?;
            } else if cmd == 0xd8 {
                println!("REQUEST: CLICK");
            } else if cmd == 0xd9 {
                let v = self.read_i8(ch)?;
                println!("REQUEST: STEP: {}", v);
            } else if cmd == 0xda {
                println!("REQUEST: TREE: GOTO ROOT");
            } else if cmd == 0xdb {
                println!("REQUEST: TREE: GO UP 1");
            } else if cmd == 0xdc {
                println!("REQUEST: DATA");
            } else if cmd == 0xdd {
                println!("REQUEST: CURRENT NODE DEFINITION");
            } else if cmd == 0xde {
                return Ok(Response::RequestEnd);
            } else if cmd == 0xdf {
                let def_len = self.read_u16(ch)? as u32;
                if def_len == 0 { let _ = self.read_u32(ch)?; }

                let parent_id     = self.read_i32(ch)?;
                let id            = self.read_i32(ch)?;
                let index         = self.read_u16(ch)?;
                let name_len      = self.read_u8(ch)?;
                let name          = self.read_string(ch, name_len as usize)?;
                let long_name_len = self.read_u8(ch)?;
                let long_name     = self.read_string(ch, long_name_len as usize)?;

                let flags = self.read_u16(ch)?;

                let node_type = match (flags >> 4) & 0x0F {
                    0 => NodeType::Node,
                    1 => NodeType::LinearFloat,
                    2 => NodeType::LogarithmicFloat,
                    3 => NodeType::FaderLevel,
                    4 => NodeType::Integer,
                    5 => NodeType::StringEnum,
                    6 => NodeType::FloatEnum,
                    7 => NodeType::String,
                    _ => NodeType::Node,
                };

                let unit = match flags & 0x0F {
                    0 => NodeUnit::None,
                    1 => NodeUnit::Db,
                    2 => NodeUnit::Percent,
                    3 => NodeUnit::Milliseconds,
                    4 => NodeUnit::Hertz,
                    5 => NodeUnit::Meters,
                    6 => NodeUnit::Seconds,
                    7 => NodeUnit::Octaves,
                    _ => NodeUnit::None,
                };

                let read_only = ((flags >> 8) & 0x01) != 0;

                let mut min_float      = Option::None;
                let mut max_float      = Option::None;
                let mut steps          = Option::None;
                let mut min_int        = Option::None;
                let mut max_int        = Option::None;
                let mut max_string_len = Option::None;
                let mut string_enum    = Option::None;
                let mut float_enum     = Option::None;

                match node_type {
                    NodeType::Node | NodeType::FaderLevel => { }
                    NodeType::String => {
                        max_string_len = Some(self.read_u16(ch)?);
                    }
                    NodeType::LinearFloat | 
                    NodeType::LogarithmicFloat => {
                        min_float = Some(self.read_f(ch)?);
                        max_float = Some(self.read_f(ch)?);
                        steps = Some(self.read_i32(ch)?);
                    }
                    NodeType::Integer => {
                        min_int = Some(self.read_i32(ch)?);
                        max_int = Some(self.read_i32(ch)?);
                    }
                    NodeType::StringEnum => {
                        let num = self.read_u16(ch)?; 
                        for _ in 0..num {
                            let item_len = self.read_u8(ch)? as usize;
                            let item = self.read_string(ch, item_len)?;
                            let long_item_len = self.read_u8(ch)? as usize;
                            let long_item = self.read_string(ch, long_item_len)?;
                            if string_enum.is_none() {
                                string_enum = Some(Vec::new());
                            }
                            string_enum.as_mut().unwrap().push(StringEnumItem {
                                item,
                                long_item,
                            });
                        }
                    }
                    NodeType::FloatEnum => {
                        let num = self.read_u16(ch)?; 
                        for _ in 0..num {
                            let item = self.read_f(ch)?;
                            let long_item_len = self.read_u8(ch)? as usize;
                            let long_item = self.read_string(ch, long_item_len)?;
                            if float_enum.is_none() {
                                float_enum = Some(Vec::new());
                            }
                            float_enum.as_mut().unwrap().push(FloatEnumItem {
                                item,
                                long_item,
                            });
                        }
                    }
                }

                let def = NodeDefinition {
                    id,
                    parent_id,
                    index,
                    name,
                    long_name,
                    node_type,
                    unit,
                    read_only,
                    min_float,
                    max_float,
                    steps,
                    min_int,
                    max_int,
                    max_string_len,
                    string_enum,
                    float_enum
                };
                return Ok(Response::NodeDefinition(def));
            }
        }
    }

    fn read_i8(&mut self, _ch:i8) -> Result<i8> {
        Ok(self.decode_next()?.1 as i8)
    }
    fn read_u8(&mut self, _ch:i8) -> Result<u8> {
        Ok(self.decode_next()?.1)
    }
    fn read_u16(&mut self, _ch:i8) -> Result<u16> {
        let a = self.decode_next()?;
        let b = self.decode_next()?;
        Ok(((a.1 as u16) << 8) | b.1 as u16)
    }
    fn read_i16(&mut self, ch:i8) -> Result<i16> {
        Ok(self.read_u16(ch)? as i16)
    }
    fn read_u32(&mut self, _ch:i8) -> Result<u32> {
        let a = self.decode_next()?;
        let b = self.decode_next()?;
        let c = self.decode_next()?;
        let d = self.decode_next()?;
        Ok(
            ((a.1 as u32) << 24) |
            ((b.1 as u32) << 16) |
            ((c.1 as u32) << 8) |
            d.1 as u32
            )
    }
    fn read_i32(&mut self, ch:i8) -> Result<i32> {
        Ok(self.read_u32(ch)? as i32)
    }

    fn read_string(&mut self, _ch:i8, len:usize) -> Result<String> {
        // define u8 array of size len and fill it with decode_next
        let buf = (0..len).map(|_| self.decode_next().map(|(_, v)| v)).collect::<Result<Vec<u8>>>()?;
        // convert u8 array to string
        String::from_utf8(buf).map_err(|_| Error::InvalidData)
    }

    fn read_f(&mut self, _ch:i8) -> Result<f32> {
        let a = self.decode_next()?;
        let b = self.decode_next()?;
        let c = self.decode_next()?;
        let d = self.decode_next()?;
        let val = ((a.1 as u32) << 24) |
            ((b.1 as u32) << 16) |
            ((c.1 as u32) << 8) |
            d.1 as u32;
        Ok(f32::from_bits(val))
    }

    fn keep_alive(&mut self) {
        if self.keep_alive_timer.elapsed() > Duration::from_secs(7) {
            self.stream.write_all(&[0xdf, 0xd1]).unwrap();
            self.keep_alive_timer = std::time::Instant::now();
        }
    }

    fn decode_next(&mut self) -> Result<(i8, u8)> {
        if self.rx_has_in_pipe.is_some() {
            // println!("has in pipe");
            let value = self.rx_has_in_pipe.unwrap();
            self.rx_has_in_pipe = None;
            return Ok((self.rx_current_channel, value));
        }

        loop {
            self.keep_alive();
            if self.rx_buf_size == 0 {
                loop {
                    match self.stream.read(&mut self.rx_buf) {
                        Ok(n) if n > 0 => {
                            // println!("got n {}...", n);
                            self.rx_buf_size = n;
                            self.rx_buf_tail = 0;
                            break;
                        }
                        // check for blocking error
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            self.keep_alive();
                            std::thread::sleep(Duration::from_millis(10));
                            continue;
                        }
                        Ok(_) => return Err(Error::ConnectionError),
                        Err(e) => return Err(e.into()),
                    }
                }
            }

            let byte = self.rx_buf[self.rx_buf_tail];
            // println!("rx_buf_tail: {}, rx_buf_size: {}, byte: {:X} buf: {}",
            //     self.rx_buf_tail,
            //     self.rx_buf_size, byte,
            //     self.rx_buf.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(","));
            self.rx_buf_tail += 1;
            self.rx_buf_size -= 1;

            if ! self.rx_esc {
                if byte == 0xdf {
                    self.rx_esc = true;
                } else {
                    break Ok((self.rx_current_channel, byte))
                }
            } else if byte == 0xdf {
                break Ok((self.rx_current_channel, byte))
            } else {
                self.rx_esc = false;
                if byte == 0xde {
                    break Ok((self.rx_current_channel, 0xdf))
                } else if (0xd0..0xde).contains(&byte) {
                    self.rx_current_channel = (byte - 0xd0) as i8;
                    continue;
                } else if self.rx_current_channel >= 0 {
                    self.rx_has_in_pipe = Some(byte);
                    break Ok((self.rx_current_channel, 0xdf))
                } else {
                    break Ok((self.rx_current_channel, byte))
                }
            }
        }
    }

    fn format_id(&self, id: i32, buf: &mut Vec<u8>, prefix: u8, suffix: Option<u8>) {
        buf.push(prefix);

        let b1 = ((id >> 24) & 0xFF) as u8;
        let b2 = ((id >> 16) & 0xFF) as u8;
        let b3 = ((id >>  8) & 0xFF) as u8;
        let b4 = ((id      ) & 0xFF) as u8;

        buf.push(b1); if b1 == 0xdf { buf.push(0xde); }
        buf.push(b2); if b2 == 0xdf { buf.push(0xde); }
        buf.push(b3); if b3 == 0xdf { buf.push(0xde); }
        buf.push(b4); if b4 == 0xdf { buf.push(0xde); }

        if let Some(suffix1) = suffix {
            buf.push(suffix1);
        }
    }

    pub fn request_node_definition(&mut self, id: i32) -> Result<()> {
        let mut buf = Vec::new();
        if id == 0 {
            buf.push(0xda);
            buf.push(0xdd);
        } else {
            self.format_id(id, &mut buf, 0xd7, Some(0xdd));
        };
        self.stream.write_all(&buf)?;
        Ok(())
    }

    pub fn request_node_data(&mut self, id: i32) -> Result<()> {
        let mut buf = Vec::new();
        if id == 0 {
            buf.push(0xda);
            buf.push(0xdc);
        } else {
            self.format_id(id, &mut buf, 0xd7, Some(0xdc));
        };
        self.stream.write_all(&buf)?;
        Ok(())
    }

    pub fn set_string(&mut self, id: i32, value: &str) -> Result<()> {
        let mut buf = Vec::new();
        self.format_id(id, &mut buf, 0xd7, None);

        if value.is_empty() {
            buf.push(0xd0);
        } else if value.len() <= 64 {
            buf.push((0x3f + value.len()).try_into().unwrap());
        } else if value.len() <= 256 {
            buf.push(0xd1);
            buf.push((value.len()-1).try_into().unwrap());
        }

        for c in value.bytes() {
            buf.push(c);
            // do we need this escaping? i guess 0xdf never really shows up in strings unless its
            // unicode stuff that the wing probably doesn't support
            // if c == 0xdf { buf.push(0xde); }
        }
        self.stream.write_all(&buf)?;
        Ok(())
    }

    pub fn set_float(&mut self, id: i32, value: f32) -> Result<()> {
        let mut buf = Vec::new();
        self.format_id(id, &mut buf, 0xd7, Some(0xd5));

        let bytes = value.to_be_bytes();
        buf.push(bytes[0]);
        buf.push(bytes[1]);
        buf.push(bytes[2]);
        buf.push(bytes[3]);

        self.stream.write_all(&buf)?;
        Ok(())
    }

    pub fn set_int(&mut self, id: i32, value: i32) -> Result<()> {
        let mut buf = Vec::new();
        self.format_id(id, &mut buf, 0xd7, None);

        let bytes = value.to_be_bytes();

        if (0..=0x3f).contains(&value) {
            buf.push(value as u8);
        } else if (-32768..=32767).contains(&value) {
            buf.push(0xd3);
            buf.push(bytes[0]);
            buf.push(bytes[1]);
        } else {
            buf.push(0xd4);
            buf.push(bytes[0]);
            buf.push(bytes[1]);
            buf.push(bytes[2]);
            buf.push(bytes[3]);
        }

        self.stream.write_all(&buf)?;
        Ok(())
    }
}

impl Drop for WingConsole {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}
