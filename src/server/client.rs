use std::net::TcpStream;

#[derive(Clone)]
pub enum UseContext {
    Global,
    Team(String),
    Channel(String, String),
    Thread(String, String, String),
}

pub struct Client {
    pub stream: TcpStream,
    pub uuid: Option<String>,
    pub use_context: UseContext,
    pub read_buffer: Vec<u8>,
    pub write_buffer: Vec<u8>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client {
            stream,
            uuid: None,
            use_context: UseContext::Global,
            read_buffer: Vec::new(),
            write_buffer: Vec::new(),
        }
    }

    pub fn extract_command(&mut self) -> Option<String> {
        if let Some(pos) = self.read_buffer.windows(1).position(|w| w == b"\n") {
            let command_bytes: Vec<u8> = self.read_buffer.drain(..pos + 1).collect();
            if let Ok(command) =
                String::from_utf8(command_bytes[..command_bytes.len() - 1].to_vec())
            {
                return Some(command);
            }
        }
        None
    }

    pub fn queue_message(&mut self, message: &str) {
        self.write_buffer.extend_from_slice(message.as_bytes());
        self.write_buffer.extend_from_slice(b"\n");
    }
}
