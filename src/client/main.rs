use std::{
    io::{BufRead, BufReader, Write, stdin},
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
};

use my_teams::ffi;

#[derive(Default)]
struct ClientState {
    uuid: String,
    name: String,
}

fn parse_status(value: &str) -> i32 {
    value.parse::<i32>().unwrap_or(0)
}

fn parse_timestamp(value: &str) -> u64 {
    value.parse::<u64>().unwrap_or(0)
}

fn handle_not_found(message: &str, parts: &[&str]) {
    let lower = message.to_ascii_lowercase();
    let identifier = parts.last().copied().unwrap_or("unknown");

    if lower.contains("thread") {
        ffi::call_client_error_unknown_thread(identifier);
    } else if lower.contains("channel") {
        ffi::call_client_error_unknown_channel(identifier);
    } else if lower.contains("team") {
        ffi::call_client_error_unknown_team(identifier);
    } else if lower.contains("user") {
        ffi::call_client_error_unknown_user(identifier);
    } else {
        println!("{message}");
    }
}

fn handle_server_message(message: &str, state: &Arc<Mutex<ClientState>>) {
    let parts: Vec<&str> = message.split('|').collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "200 Login OK" if parts.len() == 3 => {
            let uuid = parts[1];
            let name = parts[2];
            ffi::call_client_event_logged_in(uuid, name);

            if let Ok(mut s) = state.lock() {
                s.uuid = uuid.to_string();
                s.name = name.to_string();
            }
        }
        "200 Logout OK" => {
            if let Ok(s) = state.lock() {
                if !s.uuid.is_empty() && !s.name.is_empty() {
                    ffi::call_client_event_logged_out(&s.uuid, &s.name);
                }
            }

            if let Ok(mut s) = state.lock() {
                s.uuid.clear();
                s.name.clear();
            }
        }
        "EVENT PM_RECEIVED" if parts.len() == 3 => {
            ffi::call_client_event_private_message_received(parts[1], parts[2]);
        }
        "200 USERS" => {
            for user_data in parts.iter().skip(1) {
                let u_parts: Vec<&str> = user_data.split(':').collect();
                if u_parts.len() == 3 {
                    ffi::call_client_print_users(u_parts[0], u_parts[1], parse_status(u_parts[2]));
                }
            }
        }
        "200 USER" | "200 INFO_USER" if parts.len() == 4 => {
            ffi::call_client_print_user(parts[1], parts[2], parse_status(parts[3]));
        }
        "200 MESSAGES" => {
            for msg_data in parts.iter().skip(1) {
                let m_parts: Vec<&str> = msg_data.splitn(3, ':').collect();
                if m_parts.len() == 3 {
                    ffi::call_client_private_message_print_messages(
                        m_parts[0],
                        parse_timestamp(m_parts[1]),
                        m_parts[2],
                    );
                }
            }
        }
        "200 TEAM_CREATED" if parts.len() == 4 => {
            ffi::call_client_print_team_created(parts[1], parts[2], parts[3]);
        }
        "200 CHANNEL_CREATED" if parts.len() == 4 => {
            ffi::call_client_print_channel_created(parts[1], parts[2], parts[3]);
        }
        "200 THREAD_CREATED" if parts.len() == 6 => {
            ffi::call_client_print_thread_created(
                parts[1],
                parts[2],
                parse_timestamp(parts[3]),
                parts[4],
                parts[5],
            );
        }
        "200 REPLY_CREATED" if parts.len() == 5 => {
            ffi::call_client_print_reply_created(
                parts[1],
                parts[2],
                parse_timestamp(parts[3]),
                parts[4],
            );
        }
        "200 SUBSCRIBED" if parts.len() == 3 => {
            ffi::call_client_print_subscribed(parts[1], parts[2]);
        }
        "200 UNSUBSCRIBED" if parts.len() == 3 => {
            ffi::call_client_print_unsubscribed(parts[1], parts[2]);
        }
        "200 LIST_TEAMS" | "200 SUBSCRIBED_TEAMS" => {
            for team_data in parts.iter().skip(1) {
                let t_parts: Vec<&str> = team_data.splitn(3, ':').collect();
                if t_parts.len() == 3 {
                    ffi::call_client_print_teams(t_parts[0], t_parts[1], t_parts[2]);
                }
            }
        }
        "200 LIST_CHANNELS" => {
            for channel_data in parts.iter().skip(1) {
                let c_parts: Vec<&str> = channel_data.splitn(3, ':').collect();
                if c_parts.len() == 3 {
                    ffi::call_client_team_print_channels(c_parts[0], c_parts[1], c_parts[2]);
                }
            }
        }
        "200 LIST_THREADS" => {
            for thread_data in parts.iter().skip(1) {
                let t_parts: Vec<&str> = thread_data.splitn(5, ':').collect();
                if t_parts.len() == 5 {
                    ffi::call_client_channel_print_threads(
                        t_parts[0],
                        t_parts[1],
                        parse_timestamp(t_parts[2]),
                        t_parts[3],
                        t_parts[4],
                    );
                }
            }
        }
        "200 LIST_REPLIES" => {
            for reply_data in parts.iter().skip(1) {
                let r_parts: Vec<&str> = reply_data.splitn(4, ':').collect();
                if r_parts.len() == 4 {
                    ffi::call_client_thread_print_replies(
                        r_parts[0],
                        r_parts[1],
                        parse_timestamp(r_parts[2]),
                        r_parts[3],
                    );
                }
            }
        }
        "200 SUBSCRIBED_USERS" => {
            for user_data in parts.iter().skip(1) {
                let u_parts: Vec<&str> = user_data.split(':').collect();
                if u_parts.len() == 3 {
                    ffi::call_client_print_users(u_parts[0], u_parts[1], parse_status(u_parts[2]));
                }
            }
        }
        "200 INFO_TEAM" if parts.len() == 4 => {
            ffi::call_client_print_team(parts[1], parts[2], parts[3]);
        }
        "200 INFO_CHANNEL" if parts.len() == 4 => {
            ffi::call_client_print_channel(parts[1], parts[2], parts[3]);
        }
        "200 INFO_THREAD" if parts.len() == 6 => {
            ffi::call_client_print_thread(
                parts[1],
                parts[2],
                parse_timestamp(parts[3]),
                parts[4],
                parts[5],
            );
        }
        s if s.starts_with("401") => ffi::call_client_error_unauthorized(),
        s if s.starts_with("409") => ffi::call_client_error_already_exist(),
        s if s.starts_with("404") => handle_not_found(message, &parts),
        _ => println!("{message}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("USAGE: ./myteams_cli ip port");
        return;
    }

    let addr = format!("{}:{}", args[1], args[2]);
    let mut stream = TcpStream::connect(&addr).expect("Could not connect to server");
    println!("Connected to server {addr}");

    let stream_clone = stream.try_clone().expect("Error cloning TcpStream");
    let state = Arc::new(Mutex::new(ClientState::default()));
    let state_clone = Arc::clone(&state);

    thread::spawn(move || {
        let reader = BufReader::new(stream_clone);
        for line in reader.lines() {
            match line {
                Ok(message) => handle_server_message(&message, &state_clone),
                Err(_) => {
                    println!("Disconnected from server");
                    break;
                }
            }
        }
    });

    let stdin = stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(cmd) => {
                if stream.write_all(cmd.as_bytes()).is_err() {
                    println!("Failed to send command");
                    break;
                }
                if stream.write_all(b"\n").is_err() {
                    println!("Failed to send command");
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
