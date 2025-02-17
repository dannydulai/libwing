use std::collections::HashMap;
use std::net::{TcpStream, UdpSocket};
use std::io::{Read, Write};
use std::time::Duration;

use crate::{Result, Error, WingResponse};
use crate::node::{WingNodeDef, WingNodeData};
use crate::propmap::NAME_TO_DEF;

lazy_static::lazy_static! {
    static ref ID_TO_NAME: HashMap<i32, Vec<String>> = {
        let mut id2name = HashMap::<i32, Vec<String>>::new();
        if id2name.is_empty() {
            for (fullname, def) in NAME_TO_DEF.iter() {
                id2name.get_mut(&def.id).map(|x| x.push(fullname.to_string())).unwrap_or_else(|| {
                    id2name.insert(def.id, vec![fullname.to_string()]);
                });
            }
        }
        id2name
    };
}

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

    pub fn connect(host_or_ip: Option<&str>) -> Result<Self> {
        let ip =
            if let Some(i) = host_or_ip {
                i.to_string()
            } else {
                let devices = WingConsole::scan(true)?;
                if !devices.is_empty() {
                    devices[0].ip.clone()
                } else {
                    return Err(Error::DiscoveryError);
                }
            };

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

    pub fn read(&mut self) -> Result<WingResponse> {
        loop {
            let mut raw = Vec::new(); 
            let (ch, cmd) = self.decode_next(&mut raw)?;
            //println!("Channel: {}, Command: {:X}", ch, cmd);
            if cmd <= 0x3f {
                let v = cmd as i32;
                return Ok(WingResponse::NodeData(self.rx_current_channel, self.current_node_id, WingNodeData::with_i32(v)));
            } else if cmd <= 0x7f {
                let v = cmd - 0x40 + 1;
                println!("REQUEST: NODE INDEX: {}", v);
            } else if cmd <= 0xbf {
                let len = cmd - 0x80 + 1;
                let v = self.read_string(ch, len as usize, &mut raw)?;
                return Ok(WingResponse::NodeData(self.rx_current_channel, self.current_node_id, WingNodeData::with_string(v)));
            } else if cmd <= 0xcf {
                let len = cmd - 0xc0 + 1;
                let v = self.read_string(ch, len as usize, &mut raw)?;
                return Ok(WingResponse::NodeData(self.rx_current_channel, self.current_node_id, WingNodeData::with_string(v)));
            } else if cmd == 0xd0 {
                let v = String::new();
                return Ok(WingResponse::NodeData(self.rx_current_channel, self.current_node_id, WingNodeData::with_string(v)));
            } else if cmd == 0xd1 {
                let len = self.read_u8(ch, &mut raw)? + 1;
                let v = self.read_string(ch, len as usize, &mut raw)?;
                return Ok(WingResponse::NodeData(self.rx_current_channel, self.current_node_id, WingNodeData::with_string(v)));
            } else if cmd == 0xd2 {
                let v = self.read_u16(ch, &mut raw)? + 1;
                println!("REQUEST: NODE INDEX: {}", v);
            } else if cmd == 0xd3 {
                let v = self.read_i16(ch, &mut raw)?;
                return Ok(WingResponse::NodeData(self.rx_current_channel, self.current_node_id, WingNodeData::with_i16(v)));
            } else if cmd == 0xd4 {
                let v = self.read_i32(ch, &mut raw)?;
                return Ok(WingResponse::NodeData(self.rx_current_channel, self.current_node_id, WingNodeData::with_i32(v)));
            } else if cmd == 0xd5 || cmd == 0xd6 {
                let v = self.read_f(ch, &mut raw)?;
                return Ok(WingResponse::NodeData(self.rx_current_channel, self.current_node_id, WingNodeData::with_float(v)));
            } else if cmd == 0xd7 {
                self.current_node_id = self.read_i32(ch, &mut raw)?;
            } else if cmd == 0xd8 {
                println!("REQUEST: CLICK");
            } else if cmd == 0xd9 {
                let v = self.read_i8(ch, &mut raw)?;
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
                return Ok(WingResponse::RequestEnd);
            } else if cmd == 0xdf {
                let def_len = self.read_u16(ch, &mut raw)? as u32;
                if def_len == 0 { let _ = self.read_u32(ch, &mut raw)?; }
                raw.clear();
                for _ in 0..def_len { self.decode_next(&mut raw)?; } 
                return Ok(WingResponse::NodeDef(WingNodeDef::from_bytes(&raw)));
            }
        }
    }

    fn read_i8(&mut self, _ch:i8, raw: &mut Vec::<u8>) -> Result<i8> {
        Ok(self.decode_next(raw)?.1 as i8)
    }
    fn read_u8(&mut self, _ch:i8, raw: &mut Vec::<u8>) -> Result<u8> {
        Ok(self.decode_next(raw)?.1)
    }
    fn read_u16(&mut self, _ch:i8, raw: &mut Vec::<u8>) -> Result<u16> {
        let a = self.decode_next(raw)?;
        let b = self.decode_next(raw)?;
        Ok(((a.1 as u16) << 8) | b.1 as u16)
    }
    fn read_i16(&mut self, ch:i8, raw: &mut Vec::<u8>) -> Result<i16> {
        Ok(self.read_u16(ch, raw)? as i16)
    }
    fn read_u32(&mut self, _ch:i8, raw: &mut Vec::<u8>) -> Result<u32> {
        let a = self.decode_next(raw)?;
        let b = self.decode_next(raw)?;
        let c = self.decode_next(raw)?;
        let d = self.decode_next(raw)?;
        Ok(
            ((a.1 as u32) << 24) |
            ((b.1 as u32) << 16) |
            ((c.1 as u32) << 8) |
            d.1 as u32
            )
    }
    fn read_i32(&mut self, ch:i8, raw: &mut Vec::<u8>) -> Result<i32> {
        Ok(self.read_u32(ch, raw)? as i32)
    }

    fn read_string(&mut self, _ch:i8, len:usize, raw: &mut Vec::<u8>) -> Result<String> {
        // define u8 array of size len and fill it with decode_next
        let buf = (0..len).map(|_| self.decode_next(raw).map(|(_, v)| v)).collect::<Result<Vec<u8>>>()?;
        // convert u8 array to string
        String::from_utf8(buf).map_err(|_| Error::InvalidData)
    }

    fn read_f(&mut self, _ch:i8, raw: &mut Vec::<u8>) -> Result<f32> {
        let a = self.decode_next(raw)?;
        let b = self.decode_next(raw)?;
        let c = self.decode_next(raw)?;
        let d = self.decode_next(raw)?;
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

    fn decode_next(&mut self, raw: &mut Vec::<u8>) -> Result<(i8, u8)> {
        if self.rx_has_in_pipe.is_some() {
            // println!("has in pipe");
            let value = self.rx_has_in_pipe.unwrap();
            self.rx_has_in_pipe = None;
            raw.push(value);
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
                    raw.push(byte);
                    break Ok((self.rx_current_channel, byte))
                }
            } else if byte == 0xdf {
                break Ok((self.rx_current_channel, byte))
            } else {
                self.rx_esc = false;
                if byte == 0xde {
                    raw.push(0xdf);
                    break Ok((self.rx_current_channel, 0xdf))
                } else if (0xd0..0xde).contains(&byte) {
                    self.rx_current_channel = (byte - 0xd0) as i8;
                    continue;
                } else if self.rx_current_channel >= 0 {
                    self.rx_has_in_pipe = Some(byte);
                    raw.push(0xdf);
                    break Ok((self.rx_current_channel, 0xdf))
                } else {
                    raw.push(byte);
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
            buf.push(0x7f + value.len() as u8);
        } else if value.len() <= 256 {
            buf.push(0xd1);
            buf.push((value.len()-1) as u8);
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

    pub fn name_to_id(fullname: &str) -> Option<i32> {
        if let Ok(num) = fullname.parse::<i32>() {
            Some(num)
        } else {
            NAME_TO_DEF.get(fullname).map(|x| x.id)
        }
    }
    pub fn name_to_def(fullname: &str) -> Option<&WingNodeDef> {
        NAME_TO_DEF.get(fullname)
    }

    pub fn id_to_defs(id: i32) -> Option<Vec<(String, WingNodeDef)>> {
        ID_TO_NAME.get(&id)
            .cloned()
            .map(|names|
                names
                .iter()
                .map(|n| (n, NAME_TO_DEF.get(n)))
                .filter(|x| x.1.is_some())
                .map(|x| (x.0, x.1.unwrap()))
                .map(|(n, v)| (n.clone(), v.clone())
                ).collect())
    }
}

impl Drop for WingConsole {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}
