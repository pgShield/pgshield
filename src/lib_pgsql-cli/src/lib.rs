use std::io::{Read, Write};
use std::net::TcpStream;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use md5::{Md5, Digest};
use hex;

pub struct PostgresClient {
    stream: TcpStream,
}

impl PostgresClient {
    pub fn connect(host: &str, port: u16, database: &str, user: &str, password: &str) -> Result<Self, std::io::Error> {
        let mut stream = TcpStream::connect((host, port))?;
        
        // Send startup message
        let mut startup_message = Vec::new();
        startup_message.write_i32::<BigEndian>(0)?; // Placeholder for message length
        startup_message.write_i32::<BigEndian>(196608)?; // Protocol version
        startup_message.extend_from_slice(b"user\0");
        startup_message.extend_from_slice(user.as_bytes());
        startup_message.push(0);
        startup_message.extend_from_slice(b"database\0");
        startup_message.extend_from_slice(database.as_bytes());
        startup_message.push(0);
        startup_message.push(0);
        
        let length = (startup_message.len() as i32).to_be_bytes();
        startup_message[0..4].copy_from_slice(&length);
        
        stream.write_all(&startup_message)?;
        
        // Handle authentication
        loop {
            let message_type = stream.read_u8()?;
            let length = stream.read_i32::<BigEndian>()? - 4;
            let mut buffer = vec![0; length as usize];
            stream.read_exact(&mut buffer)?;
            
            match message_type {
                b'R' => {
                    let auth_type = (&buffer[..4]).read_i32::<BigEndian>()?;
                    match auth_type {
                        0 => break, // Authentication successful
                        3 => { // CleartextPassword
                            let mut response = Vec::new();
                            response.push(b'p');
                            response.write_i32::<BigEndian>(4 + password.len() as i32 + 1)?;
                            response.extend_from_slice(password.as_bytes());
                            response.push(0);
                            stream.write_all(&response)?;
                        },
                        5 => { // MD5Password
                            let salt = &buffer[4..8];
                            let hashed_password = md5_hash(user, password, salt);
                            let mut response = Vec::new();
                            response.push(b'p');
                            response.write_i32::<BigEndian>(4 + hashed_password.len() as i32 + 1)?;
                            response.extend_from_slice(hashed_password.as_bytes());
                            response.push(0);
                            stream.write_all(&response)?;
                        },
                        _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Unsupported authentication method")),
                    }
                },
                b'E' => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Authentication failed")),
                _ => {} // Ignore other message types for now
            }
        }
        
        Ok(Self { stream })
    }
    
    pub fn execute_query(&mut self, query: &str) -> Result<Vec<Vec<String>>, std::io::Error> {
        // Send query
        let mut message = Vec::new();
        message.push(b'Q');
        message.write_i32::<BigEndian>(4 + query.len() as i32 + 1)?;
        message.extend_from_slice(query.as_bytes());
        message.push(0);
        
        self.stream.write_all(&message)?;
        
        // Process response
        let mut rows = Vec::new();
        let mut columns = Vec::new();
        
        loop {
            let message_type = self.stream.read_u8()?;
            let length = self.stream.read_i32::<BigEndian>()? - 4;
            let mut buffer = vec![0; length as usize];
            self.stream.read_exact(&mut buffer)?;
            
            match message_type {
                b'T' => { // RowDescription
                    let column_count = (&buffer[..2]).read_i16::<BigEndian>()?;
                    let mut offset = 2;
                    for _ in 0..column_count {
                        let name_end = buffer[offset..].iter().position(|&x| x == 0).unwrap() + offset;
                        let name = String::from_utf8_lossy(&buffer[offset..name_end]).to_string();
                        columns.push(name);
                        offset = name_end + 18; // Skip other field information for simplicity
                    }
                },
                b'D' => { // DataRow
                    let mut row = Vec::new();
                    let field_count = (&buffer[..2]).read_i16::<BigEndian>()?;
                    let mut offset = 2;
                    for _ in 0..field_count {
                        let field_length = (&buffer[offset..offset+4]).read_i32::<BigEndian>()?;
                        offset += 4;
                        if field_length >= 0 {
                            let value = String::from_utf8_lossy(&buffer[offset..offset+field_length as usize]).to_string();
                            row.push(value);
                            offset += field_length as usize;
                        } else {
                            row.push("NULL".to_string());
                        }
                    }
                    rows.push(row);
                },
                b'C' => break, // CommandComplete
                b'E' => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Query execution failed")),
                _ => {} // Ignore other message types for simplicity
            }
        }
        
        Ok(rows)
    }
}

fn md5_hash(user: &str, password: &str, salt: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(password.as_bytes());
    hasher.update(user.as_bytes());
    let result = hasher.finalize();
    
    let mut hasher = Md5::new();
    hasher.update(hex::encode(result));
    hasher.update(salt);
    let result = hasher.finalize();
    
    format!("md5{}", hex::encode(result))
}