use std::net::{TcpStream, UdpSocket};
use std::io::{Read, Write};
use std::time::Duration;
use crate::{Result, Error};
use crate::node::{NodeDefinition, NodeData, NodeType, NodeUnit, StringEnumItem, FloatEnumItem };

const RX_BUFFER_SIZE: usize = 2048;

pub enum Response {
    NodeDefinition(NodeDefinition),
    NodeData(NodeData),
}

#[derive(Debug, Clone)]
pub struct DiscoveryInfo {
    pub ip: String,
    pub name: String,
    pub model: String,
    pub serial: String,
    pub firmware: String,
}

pub struct WingConsole {
    stream: TcpStream,
    current_node_id: i32,
    keep_alive_timer: std::time::Instant,
    rx_buf: [u8; RX_BUFFER_SIZE],
    rx_buf_tail: usize,
    rx_buf_size: usize,
    rx_esc: bool,
    rx_current_channel: i8,
    rx_has_in_pipe: Option<u8>,
    pub on_request_end: Option<Box<dyn FnMut()>>,
    pub on_node_definition: Option<Box<dyn FnMut(NodeDefinition)>>,
    pub on_node_data: Option<Box<dyn FnMut(i32, NodeData)>>,
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
                        println!("Received: {}", response);
                        let tokens: Vec<&str> = response.split(',').collect();
                        if tokens.len() >= 6 && tokens[0] == "WING" {
                            results.push(DiscoveryInfo {
                                ip: tokens[1].to_string(),
                                name: tokens[2].to_string(),
                                model: tokens[3].to_string(),
                                serial: tokens[4].to_string(),
                                firmware: tokens[5].to_string(),
                            });
                            if stop_on_first {
                                break;
                            }
                        }
                    }
                }
                Err(_) => {
                        println!("err");
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
            on_request_end: None,
            on_node_definition: None,
            on_node_data: None,
            current_node_id: 0,
            keep_alive_timer: std::time::Instant::now(),
        })
    }

    pub fn read(&mut self) -> Result<Response> {
        loop {
            let (ch, cmd) = self.decode_next()?;
            if cmd <= 0x3f {
                let v = cmd as i32;
                if let Some(ref mut cb) = self.on_node_data {
                    cb(self.current_node_id, NodeData::with_i32(v));
                }
            } else if cmd <= 0x7f {
                let v = cmd - 0x40 + 1;
                println!("REQUEST: NODE INDEX: {}", v);
            } else if cmd <= 0xbf {
                let len = cmd - 0x80 + 1;
                let v = self.read_string(len as usize)?;
                if let Some(ref mut cb) = self.on_node_data {
                    cb(self.current_node_id, NodeData::with_string(v));
                }
            } else if cmd <= 0xcf {
                let len = cmd - 0xc0 + 1;
                let v = self.read_string(len as usize)?;
                if let Some(ref mut cb) = self.on_node_data {
                    cb(self.current_node_id, NodeData::with_string(v));
                }
            } else if cmd == 0xd0 {
                let v = String::new();
                if let Some(ref mut cb) = self.on_node_data {
                    cb(self.current_node_id, NodeData::with_string(v));
                }
            } else if cmd == 0xd1 {
                let len = self.read_u8(ch)? + 1;
                let v = self.read_string(len as usize)?;
                if let Some(ref mut cb) = self.on_node_data {
                    cb(self.current_node_id, NodeData::with_string(v));
                }
            } else if cmd == 0xd2 {
                let v = self.read_u16(ch)? + 1;
                println!("REQUEST: NODE INDEX: {}", v);
            } else if cmd == 0xd3 {
                let v = self.read_i16(ch)?;
                if let Some(ref mut cb) = self.on_node_data {
                    cb(self.current_node_id, NodeData::with_i16(v));
                }
            } else if cmd == 0xd4 {
                let v = self.read_i32(ch)?;
                if let Some(ref mut cb) = self.on_node_data {
                    cb(self.current_node_id, NodeData::with_i32(v));
                }
            } else if cmd == 0xd5 || cmd == 0xd6 {
                let v = self.read_f(ch)?;
                if let Some(ref mut cb) = self.on_node_data {
                    cb(self.current_node_id, NodeData::with_float(v));
                }
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
                if let Some(ref mut cb) = self.on_request_end {
                    cb();
                }
            } else if cmd == 0xdf {
            }

        }
        // let mut buf = [0u8; 1024];
        // println!("Reading...");
        //
        //
        // match self.stream.read(&mut buf) {
        //     Ok(n) if n > 0 => {
        //         println!("got n {}...", n);
        //         // Copy new data to rx buffer
        //         if self.rx_buf_size + n <= RX_BUFFER_SIZE {
        //             self.rx_buf[self.rx_buf_size..self.rx_buf_size + n]
        //                 .copy_from_slice(&buf[..n]);
        //             self.rx_buf_size += n;
        //         }
        //
        //         // Process received data
        //         while self.rx_buf_size > 0 {
        //             let (channel, value) = self.decode_next()?;
        //             // print channel and value
        //             println!("Channel: {}, Value: {}", channel, value);
        //             if value != 0 {
        //                 match channel {
        //                     0 => if let Some(ref mut cb) = self.on_request_end {
        //                         cb();
        //                     },
        //                     1 => {
        //                         // Handle node definition data
        //                         if let Some(def) = self.accumulate_node_definition(value)? {
        //                             if let Some(ref mut cb) = self.on_node_definition {
        //                                 cb(def);
        //                             }
        //                         }
        //                     },
        //                     2 => {
        //                         // Handle node data
        //                         if let Some((id, data)) = self.accumulate_node_data(value)? {
        //                             if let Some(ref mut cb) = self.on_node_data {
        //                                 cb(id, data);
        //                             }
        //                         }
        //                     },
        //                     _ => {}
        //                 }
        //             }
        //         }
        //
        //         Ok(())
        //     }
        //     Ok(_) => Err(Error::ConnectionError),
        //     Err(e) => Err(e.into()),
        // }
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

    fn read_string(&mut self, len:usize) -> Result<String> {
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
                            println!("got n {}...", n);
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

    pub fn request_node_definition(&mut self, id: i32) -> Result<()> {
        let mut buf = [0u8; 16];
        let len = if id == 0 {
            buf[0] = 0xda;
            buf[1] = 0xdd;
            2
        } else {
            buf[0] = 0xd7;
            buf[1] = (id >> 24) as u8;
            buf[2] = ((id >> 16) & 0xff) as u8;
            buf[3] = ((id >> 8) & 0xff) as u8;
            buf[4] = (id & 0xff) as u8;
            buf[5] = 0xdd;
            6
        };
        self.stream.write_all(&buf[..len])?;
        Ok(())
    }

    pub fn scan_children(&mut self, id: i32) -> Result<()> {
        // Request definition for this node
        self.request_node_definition(id)?;
        
        // Request data for this node
        self.request_node_data(id)?;
        
        // Request definitions for all child nodes
        for i in 1..=64 {  // Assuming max 64 children per node
            let child_id = (id << 8) | i;
            self.request_node_definition(child_id)?;
        }
        
        Ok(())
    }

    pub fn request_node_data(&mut self, id: i32) -> Result<()> {
        let mut buf = [0u8; 16];
        let len = if id == 0 {
            buf[0] = 0xda;
            buf[1] = 0xdc;
            2
        } else {
            buf[0] = 0xd7;
            buf[1] = (id >> 24) as u8;
            buf[2] = ((id >> 16) & 0xff) as u8;
            buf[3] = ((id >> 8) & 0xff) as u8;
            buf[4] = (id & 0xff) as u8;
            buf[5] = 0xdc;
            6
        };
        self.stream.write_all(&buf[..len])?;
        Ok(())
    }


    // fn accumulate_node_definition(&mut self, value: u8) -> Result<Option<NodeDefinition>> {
    //     self.node_def_buffer.push(value);
    //     
    //     // Check if we have a complete definition
    //     if value == b'\n' {
    //         let def_str = String::from_utf8_lossy(&self.node_def_buffer);
    //         let parts: Vec<&str> = def_str.trim().split(',').collect();
    //         
    //         if parts.len() >= 12 {
    //             let def = NodeDefinition {
    //                 id: u32::from_str_radix(parts[0], 16).unwrap_or(0),
    //                 parent_id: u32::from_str_radix(parts[1], 16).unwrap_or(0),
    //                 index: parts[2].parse().unwrap_or(0),
    //                 name: parts[3].to_string(),
    //                 long_name: parts[4].to_string(),
    //                 node_type: match parts[5].parse().unwrap_or(0) {
    //                     0 => NodeType::Node,
    //                     1 => NodeType::LinearFloat,
    //                     2 => NodeType::LogarithmicFloat,
    //                     3 => NodeType::FaderLevel,
    //                     4 => NodeType::Integer,
    //                     5 => NodeType::StringEnum,
    //                     6 => NodeType::FloatEnum,
    //                     7 => NodeType::String,
    //                     _ => NodeType::Node,
    //                 },
    //                 unit: match parts[6].parse().unwrap_or(0) {
    //                     0 => NodeUnit::None,
    //                     1 => NodeUnit::Db,
    //                     2 => NodeUnit::Percent,
    //                     3 => NodeUnit::Milliseconds,
    //                     4 => NodeUnit::Hertz,
    //                     5 => NodeUnit::Meters,
    //                     6 => NodeUnit::Seconds,
    //                     7 => NodeUnit::Octaves,
    //                     _ => NodeUnit::None,
    //                 },
    //                 read_only: parts[7] == "1",
    //                 min_float: parts[8].parse().unwrap_or(0.0),
    //                 max_float: parts[9].parse().unwrap_or(0.0),
    //                 steps: parts[10].parse().unwrap_or(0),
    //                 min_int: parts[11].parse().unwrap_or(0),
    //                 max_int: parts[12].parse().unwrap_or(0),
    //                 max_string_len: parts[13].parse().unwrap_or(0),
    //                 string_enum: if parts.len() > 14 {
    //                     parts[14].split(';')
    //                         .filter_map(|pair| {
    //                             let items: Vec<&str> = pair.split('=').collect();
    //                             if items.len() == 2 {
    //                                 Some(StringEnumItem {
    //                                     item: items[0].to_string(),
    //                                     longitem: items[1].to_string(),
    //                                 })
    //                             } else {
    //                                 None
    //                             }
    //                         })
    //                         .collect()
    //                 } else {
    //                     Vec::new()
    //                 },
    //                 float_enum: if parts.len() > 15 {
    //                     parts[15].split(';')
    //                         .filter_map(|pair| {
    //                             let items: Vec<&str> = pair.split('=').collect();
    //                             if items.len() == 2 {
    //                                 if let Ok(val) = items[0].parse() {
    //                                     Some(FloatEnumItem {
    //                                         item: val,
    //                                         longitem: items[1].to_string(),
    //                                     })
    //                                 } else {
    //                                     None
    //                                 }
    //                             } else {
    //                                 None
    //                             }
    //                         })
    //                         .collect()
    //                 } else {
    //                     Vec::new()
    //                 },
    //             };
    //             
    //             self.node_def_buffer.clear();
    //             return Ok(Some(def));
    //         }
    //         
    //         self.node_def_buffer.clear();
    //     }
    //     
    //     Ok(None)
    // }
    //
    // fn accumulate_node_data(&mut self, value: u8) -> Result<Option<(u32, NodeData)>> {
    //     self.node_data_buffer.push(value);
    //     
    //     // Check if we have a complete data packet
    //     if value == b'\n' {
    //         let data_str = String::from_utf8_lossy(&self.node_data_buffer);
    //         let parts: Vec<&str> = data_str.trim().split(',').collect();
    //         
    //         if parts.len() >= 2 {
    //             let id = u32::from_str_radix(parts[0], 16).unwrap_or(0);
    //             let data = match parts[1].chars().next() {
    //                 Some('S') => NodeData::with_string(parts[2..].join(",").trim().to_string()),
    //                 Some('F') => NodeData::with_float(parts[2].parse().unwrap_or(0.0)),
    //                 Some('I') => NodeData::with_int(parts[2].parse().unwrap_or(0)),
    //                 _ => NodeData::new(),
    //             };
    //             
    //             self.node_data_buffer.clear();
    //             return Ok(Some((id, data)));
    //         }
    //         
    //         self.node_data_buffer.clear();
    //     }
    //     
    //     Ok(None)
    // }

    fn format_id(&self, id: i32, buf: &mut [u8], prefix: u8, suffix: u8) -> Result<()> {
        buf[0] = prefix;
        // Format ID as 6 hex digits
        for i in 0..6 {
            let digit = ((id >> (20 - 4 * i)) & 0xF) as u8;
            buf[i + 1] = if digit < 10 {
                b'0' + digit
            } else {
                b'A' + (digit - 10)
            };
        }
        buf[7] = suffix;
        Ok(())
    }

    pub fn set_string(&mut self, id: i32, value: &str) -> Result<()> {
        let mut buf = Vec::with_capacity(8 + value.len() + 1);
        self.format_id(id, &mut buf[..8], b'S', b' ')?;
        buf.extend_from_slice(value.as_bytes());
        buf.push(b'\n');
        self.stream.write_all(&buf)?;
        Ok(())
    }

    pub fn set_float(&mut self, id: i32, value: f32) -> Result<()> {
        let mut buf = [0u8; 32];
        self.format_id(id, &mut buf[..8], b'F', b' ')?;
        let value_str = format!("{}", value);
        let value_bytes = value_str.as_bytes();
        buf[8..8+value_bytes.len()].copy_from_slice(value_bytes);
        buf[8+value_bytes.len()] = b'\n';
        self.stream.write_all(&buf[..8+value_bytes.len()+1])?;
        Ok(())
    }

    pub fn set_int(&mut self, id: i32, value: i32) -> Result<()> {
        let mut buf = [0u8; 32];
        self.format_id(id, &mut buf[..8], b'I', b' ')?;
        let value_str = format!("{}", value);
        let value_bytes = value_str.as_bytes();
        buf[8..8+value_bytes.len()].copy_from_slice(value_bytes);
        buf[8+value_bytes.len()] = b'\n';
        self.stream.write_all(&buf[..8+value_bytes.len()+1])?;
        Ok(())
    }
}

impl Drop for WingConsole {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}
