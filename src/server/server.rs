use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    net::{SocketAddr, TcpListener},
    sync::atomic::Ordering,
};

use my_teams::ffi;

use crate::{client::Client, models::Database};

pub struct Server {
    pub(crate) listener: TcpListener,
    pub(crate) clients: HashMap<SocketAddr, Client>,
    pub(crate) db: Database,
}

impl Server {
    pub fn new(port: u16) -> std::io::Result<Self> {
        let address = format!("0.0.0.0:{port}");
        let listener = TcpListener::bind(&address)?;
        listener.set_nonblocking(true)?;

        let mut db = Database::default();
        db.load_from_file("myteams.data").ok();

        Ok(Server {
            listener,
            clients: HashMap::new(),
            db,
        })
    }

    fn accept_new_clients(&mut self) {
        match self.listener.accept() {
            Ok((stream, addr)) => {
                if stream.set_nonblocking(true).is_ok() {
                    self.clients.insert(addr, Client::new(stream));
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => println!("Error accepting new client: {e}"),
        }
    }

    fn handle_command(&mut self, addr: SocketAddr, command_line: &str) {
        let args = match Self::parse_command_args(command_line) {
            Ok(args) => args,
            Err(_) => {
                self.send_to(addr, "400 Bad Request");
                return;
            }
        };

        if args.is_empty() {
            return;
        }

        let command = args[0].as_str();

        match command {
            "/help" => self.cmd_help(addr),
            "/login" => self.cmd_login(addr, &args),
            "/logout" => self.cmd_logout(addr),
            "/send" => self.cmd_send(addr, &args),
            "/use" => self.cmd_use(addr, &args),
            "/users" => self.cmd_users(addr),
            "/user" => self.cmd_user(addr, &args),
            "/messages" => self.cmd_messages(addr, &args),
            "/subscribe" => self.cmd_subscribe(addr, &args),
            "/unsubscribe" => self.cmd_unsubscribe(addr, &args),
            "/create" => self.cmd_create(addr, &args),
            "/list" => self.cmd_list(addr),
            "/info" => self.cmd_info(addr),
            "/subscribed" => self.cmd_subscribed(addr, &args),
            _ => {
                if let Some(client) = self.clients.get_mut(&addr) {
                    client.queue_message("400 Unknown Command");
                }
            }
        }
    }

    fn process_clients(&mut self) {
        let mut disconnected = Vec::new();
        let mut commands_to_process = Vec::new();

        for (addr, client) in self.clients.iter_mut() {
            let mut buffer = [0; 2048];
            match client.stream.read(&mut buffer) {
                Ok(0) => disconnected.push(*addr),
                Ok(n) => {
                    client.read_buffer.extend_from_slice(&buffer[..n]);
                    while let Some(cmd) = client.extract_command() {
                        commands_to_process.push((*addr, cmd));
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(_) => disconnected.push(*addr),
            }
        }

        for (addr, cmd) in commands_to_process {
            self.handle_command(addr, &cmd);
        }

        for (addr, client) in self.clients.iter_mut() {
            if !client.write_buffer.is_empty() {
                match client.stream.write(&client.write_buffer) {
                    Ok(n) => {
                        client.write_buffer.drain(..n);
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                    Err(_) => disconnected.push(*addr),
                }
            }
        }

        for addr in disconnected {
            if let Some(client) = self.clients.remove(&addr) {
                if let Some(uuid) = client.uuid {
                    ffi::call_user_logged_out(&uuid);
                }
                let _ = client.stream.shutdown(std::net::Shutdown::Both);
            }
        }
    }

    pub fn run(&mut self) {
        println!("Server listening...");

        while my_teams::ffi::RUNNING.load(Ordering::SeqCst) {
            self.accept_new_clients();
            self.process_clients();

            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        println!("\nShutting down server. Saving data...");
        if let Err(e) = self.db.save_to_file("myteams.data") {
            println!("Error saving data: {e}");
        }
    }

    pub(crate) fn parse_command_args(command_line: &str) -> Result<Vec<String>, String> {
        let line = command_line.trim();
        if line.is_empty() {
            return Ok(Vec::new());
        }

        let bytes = line.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        let command_start = i;
        while i < len && !bytes[i].is_ascii_whitespace() {
            if bytes[i] == b'"' {
                return Err("command must not be quoted".to_string());
            }
            i += 1;
        }

        if command_start == i {
            return Err("missing command".to_string());
        }

        let mut args = vec![line[command_start..i].to_string()];

        while i < len {
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            if i >= len {
                break;
            }

            if bytes[i] != b'"' {
                return Err("arguments must be quoted".to_string());
            }

            i += 1;
            let arg_start = i;

            while i < len && bytes[i] != b'"' {
                i += 1;
            }

            if i >= len {
                return Err("missing closing quote".to_string());
            }

            args.push(line[arg_start..i].to_string());
            i += 1;

            if i < len && !bytes[i].is_ascii_whitespace() {
                return Err("unexpected characters after quoted argument".to_string());
            }
        }

        Ok(args)
    }

    pub(crate) fn send_to(&mut self, addr: SocketAddr, msg: &str) {
        if let Some(client) = self.clients.get_mut(&addr) {
            client.queue_message(msg);
        }
    }

    pub(crate) fn get_client_uuid(&self, addr: SocketAddr) -> Option<String> {
        self.clients.get(&addr)?.uuid.clone()
    }
}
