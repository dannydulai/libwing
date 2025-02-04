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
                    // Handle decoded data...
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

        let mut channel = -1;
        let mut value = 0u8;

        // Implement protocol decoding here...

        Ok((channel, value))
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

    fn format_id(&self, id: u32, buf: &mut [u8], prefix: u8, suffix: u8) -> Result<()> {
        buf[0] = prefix;
        // Format ID as hex...
        buf[7] = suffix;
        Ok(())
    }

    pub fn set_string(&mut self, id: u32, value: &str) -> Result<()> {
        // Implement string setting
        Ok(())
    }

    pub fn set_float(&mut self, id: u32, value: f32) -> Result<()> {
        // Implement float setting
        Ok(())
    }

    pub fn set_int(&mut self, id: u32, value: i32) -> Result<()> {
        // Implement int setting
        Ok(())
    }
}

impl Drop for WingConsole {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}
