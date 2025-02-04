use std::net::{TcpStream, UdpSocket};
use std::io::{Read, Write};
use std::time::Duration;
use crate::{Result, Error};
use crate::node::{NodeDefinition, NodeData};

const RX_BUFFER_SIZE: usize = 2048;

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
    rx_buf: [u8; RX_BUFFER_SIZE],
    rx_buf_tail: usize,
    rx_buf_size: usize,
    rx_esc: bool,
    rx_current_channel: i32,
    rx_has_in_pipe: bool,
    pub on_request_end: Option<Box<dyn FnMut()>>,
    pub on_node_definition: Option<Box<dyn FnMut(NodeDefinition)>>,
    pub on_node_data: Option<Box<dyn FnMut(u32, NodeData)>>,
    node_def_buffer: Vec<u8>,
    node_data_buffer: Vec<u8>,
    current_node_id: u32,
}

impl WingConsole {
    pub fn scan(stop_on_first: bool) -> Vec<DiscoveryInfo> {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_broadcast(true).unwrap();
        socket.set_read_timeout(Some(Duration::from_millis(100))).unwrap();

        let _ = socket.send_to(b"WING", "255.255.255.255:2345");

        let mut results = Vec::new();
        let mut buf = [0u8; 1024];
        let mut attempts = 0;

        while attempts < 10 {
            match socket.recv_from(&mut buf) {
                Ok((received, _)) => {
                    if let Ok(response) = String::from_utf8(buf[..received].to_vec()) {
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
                    attempts += 1;
                }
            }
        }

        results
    }

    pub fn connect(ip: &str) -> Result<Self> {
        let stream = TcpStream::connect((ip, 2345))?;
        stream.set_nodelay(true)?;
        
        Ok(Self {
            stream,
            rx_buf: [0; RX_BUFFER_SIZE],
            rx_buf_tail: 0,
            rx_buf_size: 0,
            rx_esc: false,
            rx_current_channel: -1,
            rx_has_in_pipe: false,
            on_request_end: None,
            on_node_definition: None,
            on_node_data: None,
            node_def_buffer: Vec::new(),
            node_data_buffer: Vec::new(),
            current_node_id: 0,
        })
    }

    pub fn read(&mut self) -> Result<()> {
        let mut buf = [0u8; 1024];
        match self.stream.read(&mut buf) {
            Ok(n) if n > 0 => {
                // Copy new data to rx buffer
                if self.rx_buf_size + n <= RX_BUFFER_SIZE {
                    self.rx_buf[self.rx_buf_size..self.rx_buf_size + n]
                        .copy_from_slice(&buf[..n]);
                    self.rx_buf_size += n;
                }
                
                // Process received data
                while self.rx_buf_size > 0 {
                    let (channel, value) = self.decode_next()?;
                    if value != 0 {
                        match channel {
                            0 => if let Some(ref mut cb) = self.on_request_end {
                                cb();
                            },
                            1 => {
                                // Handle node definition data
                                if let Some(def) = self.accumulate_node_definition(value)? {
                                    if let Some(ref mut cb) = self.on_node_definition {
                                        cb(def);
                                    }
                                }
                            },
                            2 => {
                                // Handle node data
                                if let Some((id, data)) = self.accumulate_node_data(value)? {
                                    if let Some(ref mut cb) = self.on_node_data {
                                        cb(id, data);
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                }
                
                Ok(())
            }
            Ok(_) => Err(Error::ConnectionError),
            Err(e) => Err(e.into()),
        }
    }

    fn decode_next(&mut self) -> Result<(i32, u8)> {
        if self.rx_buf_size == 0 {
            return Err(Error::InvalidData);
        }

        let byte = self.rx_buf[self.rx_buf_tail];
        self.rx_buf_tail = (self.rx_buf_tail + 1) % RX_BUFFER_SIZE;
        self.rx_buf_size -= 1;

        if self.rx_esc {
            self.rx_esc = false;
            Ok((self.rx_current_channel, byte ^ 0x20))
        } else if byte == 0x1B {
            self.rx_esc = true;
            Ok((self.rx_current_channel, 0))
        } else if byte >= 0x20 {
            self.rx_current_channel = (byte & 0x1F) as i32;
            self.rx_has_in_pipe = (byte & 0x20) != 0;
            Ok((self.rx_current_channel, 0))
        } else {
            Ok((self.rx_current_channel, byte))
        }
    }

    pub fn request_node_definition(&mut self, id: u32) -> Result<()> {
        // Format and send node definition request
        let mut buf = [0u8; 8];
        self.format_id(id, &mut buf, b'D', b'\n')?;
        self.stream.write_all(&buf)?;
        Ok(())
    }

    pub fn request_node_data(&mut self, id: u32) -> Result<()> {
        // Format and send node data request
        let mut buf = [0u8; 8];
        self.format_id(id, &mut buf, b'R', b'\n')?;
        self.stream.write_all(&buf)?;
        Ok(())
    }


    fn accumulate_node_definition(&mut self, value: u8) -> Result<Option<NodeDefinition>> {
        self.node_def_buffer.push(value);
        
        // Check if we have a complete definition
        if value == b'\n' {
            let def_str = String::from_utf8_lossy(&self.node_def_buffer);
            let parts: Vec<&str> = def_str.trim().split(',').collect();
            
            if parts.len() >= 12 {
                let def = NodeDefinition {
                    id: u32::from_str_radix(parts[0], 16).unwrap_or(0),
                    parent_id: u32::from_str_radix(parts[1], 16).unwrap_or(0),
                    index: parts[2].parse().unwrap_or(0),
                    name: parts[3].to_string(),
                    long_name: parts[4].to_string(),
                    node_type: match parts[5].parse().unwrap_or(0) {
                        0 => NodeType::Node,
                        1 => NodeType::LinearFloat,
                        2 => NodeType::LogarithmicFloat,
                        3 => NodeType::FaderLevel,
                        4 => NodeType::Integer,
                        5 => NodeType::StringEnum,
                        6 => NodeType::FloatEnum,
                        7 => NodeType::String,
                        _ => NodeType::Node,
                    },
                    unit: match parts[6].parse().unwrap_or(0) {
                        0 => NodeUnit::None,
                        1 => NodeUnit::Db,
                        2 => NodeUnit::Percent,
                        3 => NodeUnit::Milliseconds,
                        4 => NodeUnit::Hertz,
                        5 => NodeUnit::Meters,
                        6 => NodeUnit::Seconds,
                        7 => NodeUnit::Octaves,
                        _ => NodeUnit::None,
                    },
                    read_only: parts[7] == "1",
                    min_float: parts[8].parse().unwrap_or(0.0),
                    max_float: parts[9].parse().unwrap_or(0.0),
                    steps: parts[10].parse().unwrap_or(0),
                    min_int: parts[11].parse().unwrap_or(0),
                    max_int: parts[12].parse().unwrap_or(0),
                    max_string_len: parts[13].parse().unwrap_or(0),
                    string_enum: Vec::new(), // TODO: Parse enum items
                    float_enum: Vec::new(),  // TODO: Parse enum items
                };
                
                self.node_def_buffer.clear();
                return Ok(Some(def));
            }
            
            self.node_def_buffer.clear();
        }
        
        Ok(None)
    }

    fn accumulate_node_data(&mut self, value: u8) -> Result<Option<(u32, NodeData)>> {
        self.node_data_buffer.push(value);
        
        // Check if we have a complete data packet
        if value == b'\n' {
            let data_str = String::from_utf8_lossy(&self.node_data_buffer);
            let parts: Vec<&str> = data_str.trim().split(',').collect();
            
            if parts.len() >= 2 {
                let id = u32::from_str_radix(parts[0], 16).unwrap_or(0);
                let data = match parts[1].chars().next() {
                    Some('S') => NodeData::with_string(parts[2..].join(",").trim().to_string()),
                    Some('F') => NodeData::with_float(parts[2].parse().unwrap_or(0.0)),
                    Some('I') => NodeData::with_int(parts[2].parse().unwrap_or(0)),
                    _ => NodeData::new(),
                };
                
                self.node_data_buffer.clear();
                return Ok(Some((id, data)));
            }
            
            self.node_data_buffer.clear();
        }
        
        Ok(None)
    }

    fn format_id(&self, id: u32, buf: &mut [u8], prefix: u8, suffix: u8) -> Result<()> {
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

    pub fn set_string(&mut self, id: u32, value: &str) -> Result<()> {
        let mut buf = Vec::with_capacity(8 + value.len() + 1);
        self.format_id(id, &mut buf[..8], b'S', b' ')?;
        buf.extend_from_slice(value.as_bytes());
        buf.push(b'\n');
        self.stream.write_all(&buf)?;
        Ok(())
    }

    pub fn set_float(&mut self, id: u32, value: f32) -> Result<()> {
        let mut buf = [0u8; 32];
        self.format_id(id, &mut buf[..8], b'F', b' ')?;
        let value_str = format!("{}", value);
        let value_bytes = value_str.as_bytes();
        buf[8..8+value_bytes.len()].copy_from_slice(value_bytes);
        buf[8+value_bytes.len()] = b'\n';
        self.stream.write_all(&buf[..8+value_bytes.len()+1])?;
        Ok(())
    }

    pub fn set_int(&mut self, id: u32, value: i32) -> Result<()> {
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
