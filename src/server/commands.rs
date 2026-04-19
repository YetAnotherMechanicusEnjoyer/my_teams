use std::{
    collections::HashMap,
    net::SocketAddr,
    time::UNIX_EPOCH,
};

use my_teams::ffi;

use crate::{
    client::UseContext,
    models::{generate_uuid, Channel, Message, Team, Thread, User},
    server::Server,
};

const MAX_NAME_LENGTH: usize = 32;
const MAX_DESCRIPTION_LENGTH: usize = 255;
const MAX_BODY_LENGTH: usize = 512;

impl Server {
    pub(crate) fn cmd_login(&mut self, addr: SocketAddr, args: &[String]) {
        if args.len() != 2 {
            self.send_to(addr, "400 Bad Request: Missing user_name");
            return;
        }

        let user_name = &args[1];
        if user_name.len() > MAX_NAME_LENGTH {
            self.send_to(addr, "400 Bad Request: Name too long");
            return;
        }

        let existing_uuid = self
            .db
            .users
            .values()
            .find(|u| u.name == *user_name)
            .map(|u| u.uuid.clone());

        let user_uuid = match existing_uuid {
            Some(uuid) => {
                if let Some(user) = self.db.users.get_mut(&uuid) {
                    user.is_connected = true;
                }
                uuid
            }
            None => {
                let uuid = generate_uuid();
                let new_user = User {
                    uuid: uuid.clone(),
                    name: user_name.clone(),
                    is_connected: true,
                };
                self.db.users.insert(uuid.clone(), new_user);

                ffi::call_user_created(&uuid, user_name);
                uuid
            }
        };

        if let Some(client) = self.clients.get_mut(&addr) {
            client.uuid = Some(user_uuid.clone());
            client.queue_message(&format!("200 Login OK|{user_uuid}|{user_name}"));
        }

        ffi::call_user_logged_in(&user_uuid);
    }

    pub(crate) fn cmd_logout(&mut self, addr: SocketAddr) {
        if let Some(client) = self.clients.get_mut(&addr) {
            if let Some(uuid) = &client.uuid {
                ffi::call_user_logged_out(uuid);

                if let Some(user) = self.db.users.get_mut(uuid) {
                    user.is_connected = false;
                }
            }
            client.uuid = None;
            client.use_context = UseContext::Global;
            client.queue_message("200 Logout OK");
        }
    }

    pub(crate) fn cmd_send(&mut self, addr: SocketAddr, args: &[String]) {
        if args.len() != 3 {
            self.send_to(addr, "400 Bad Request: /send \"user_uuid\" \"message\"");
            return;
        }

        let sender_uuid = match self.get_client_uuid(addr) {
            Some(uuid) => uuid,
            None => {
                self.send_to(addr, "401 Unauthorized: Please login first");
                return;
            }
        };

        let target_uuid = &args[1];
        let message_body = &args[2];

        if message_body.len() > MAX_BODY_LENGTH {
            self.send_to(addr, "400 Bad Request: Message too long");
            return;
        }

        if !self.db.users.contains_key(target_uuid) {
            self.send_to(addr, "404 Not Found: User does not exist");
            return;
        }

        let msg = Message {
            sender_uuid: sender_uuid.clone(),
            receiver_uuid: target_uuid.clone(),
            body: message_body.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        self.db.private_messages.push(msg);

        ffi::call_private_message_sended(&sender_uuid, target_uuid, message_body);

        self.send_event_to_user(
            target_uuid,
            &format!("EVENT PM_RECEIVED|{sender_uuid}|{message_body}"),
        );

        self.send_to(addr, "200 Message Sent");
    }

    pub(crate) fn cmd_use(&mut self, addr: SocketAddr, args: &[String]) {
        let client_uuid = match self.get_client_uuid(addr) {
            Some(uuid) => uuid,
            None => {
                self.send_to(addr, "401 Unauthorized: Please login first");
                return;
            }
        };

        match args.len() {
            1 => {
                if let Some(client) = self.clients.get_mut(&addr) {
                    client.use_context = UseContext::Global;
                    client.queue_message("200 Context Updated");
                }
            }
            2 => {
                let team_uuid = &args[1];
                let team = match self.db.teams.get(team_uuid) {
                    Some(team) => team,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Team");
                        return;
                    }
                };

                if !team.subscribers.iter().any(|u| u == &client_uuid) {
                    self.send_to(addr, "401 Unauthorized: Not subscribed to team");
                    return;
                }

                if let Some(client) = self.clients.get_mut(&addr) {
                    client.use_context = UseContext::Team(team_uuid.clone());
                    client.queue_message("200 Context Updated");
                }
            }
            3 => {
                let team_uuid = &args[1];
                let channel_uuid = &args[2];

                let team = match self.db.teams.get(team_uuid) {
                    Some(team) => team,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Team");
                        return;
                    }
                };

                if !team.subscribers.iter().any(|u| u == &client_uuid) {
                    self.send_to(addr, "401 Unauthorized: Not subscribed to team");
                    return;
                }

                if !team.channels.contains_key(channel_uuid) {
                    self.send_to(addr, "404 Not Found: Unknown Channel");
                    return;
                }

                if let Some(client) = self.clients.get_mut(&addr) {
                    client.use_context = UseContext::Channel(team_uuid.clone(), channel_uuid.clone());
                    client.queue_message("200 Context Updated");
                }
            }
            4 => {
                let team_uuid = &args[1];
                let channel_uuid = &args[2];
                let thread_uuid = &args[3];

                let team = match self.db.teams.get(team_uuid) {
                    Some(team) => team,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Team");
                        return;
                    }
                };

                if !team.subscribers.iter().any(|u| u == &client_uuid) {
                    self.send_to(addr, "401 Unauthorized: Not subscribed to team");
                    return;
                }

                let channel = match team.channels.get(channel_uuid) {
                    Some(channel) => channel,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Channel");
                        return;
                    }
                };

                if !channel.threads.contains_key(thread_uuid) {
                    self.send_to(addr, "404 Not Found: Unknown Thread");
                    return;
                }

                if let Some(client) = self.clients.get_mut(&addr) {
                    client.use_context = UseContext::Thread(
                        team_uuid.clone(),
                        channel_uuid.clone(),
                        thread_uuid.clone(),
                    );
                    client.queue_message("200 Context Updated");
                }
            }
            _ => self.send_to(addr, "400 Bad Request: Invalid /use arguments"),
        }
    }

    pub(crate) fn cmd_users(&mut self, addr: SocketAddr) {
        if self.get_client_uuid(addr).is_none() {
            self.send_to(addr, "401 Unauthorized: Please login first");
            return;
        }

        let mut response = String::from("200 USERS");
        for user in self.db.users.values() {
            let status = if user.is_connected { "1" } else { "0" };
            response.push_str(&format!("|{}:{}:{status}", user.uuid, user.name));
        }
        self.send_to(addr, &response);
    }

    pub(crate) fn cmd_user(&mut self, addr: SocketAddr, args: &[String]) {
        if args.len() != 2 {
            self.send_to(addr, "400 Bad Request");
            return;
        }
        if self.get_client_uuid(addr).is_none() {
            self.send_to(addr, "401 Unauthorized");
            return;
        }

        let target_uuid = &args[1];
        match self.db.users.get(target_uuid) {
            Some(user) => {
                let status = if user.is_connected { "1" } else { "0" };
                self.send_to(
                    addr,
                    &format!("200 USER|{}|{}|{status}", user.uuid, user.name),
                );
            }
            None => self.send_to(addr, "404 Not Found: User not found"),
        }
    }

    pub(crate) fn cmd_messages(&mut self, addr: SocketAddr, args: &[String]) {
        if args.len() != 2 {
            self.send_to(addr, "400 Bad Request");
            return;
        }
        let client_uuid = match self.get_client_uuid(addr) {
            Some(uuid) => uuid,
            None => {
                self.send_to(addr, "401 Unauthorized");
                return;
            }
        };

        let target_uuid = &args[1];
        if !self.db.users.contains_key(target_uuid) {
            self.send_to(addr, "404 Not Found: User not found");
            return;
        }

        let mut response = String::from("200 MESSAGES");
        for msg in &self.db.private_messages {
            if (msg.sender_uuid == client_uuid && msg.receiver_uuid == *target_uuid)
                || (msg.sender_uuid == *target_uuid && msg.receiver_uuid == client_uuid)
            {
                response.push_str(&format!(
                    "|{}:{}:{}",
                    msg.sender_uuid, msg.timestamp, msg.body
                ));
            }
        }
        self.send_to(addr, &response);
    }

    pub(crate) fn cmd_subscribe(&mut self, addr: SocketAddr, args: &[String]) {
        if args.len() != 2 {
            self.send_to(addr, "400 Bad Request");
            return;
        }
        let client_uuid = match self.get_client_uuid(addr) {
            Some(uuid) => uuid,
            None => {
                self.send_to(addr, "401 Unauthorized");
                return;
            }
        };

        let team_uuid = &args[1];
        if let Some(team) = self.db.teams.get_mut(team_uuid) {
            if !team.subscribers.iter().any(|u| u == &client_uuid) {
                team.subscribers.push(client_uuid.clone());
            }
            ffi::call_user_subscribed(team_uuid, &client_uuid);
            self.send_to(addr, &format!("200 SUBSCRIBED|{client_uuid}|{team_uuid}"));
        } else {
            self.send_to(addr, "404 Not Found: Team not found");
        }
    }

    pub(crate) fn cmd_unsubscribe(&mut self, addr: SocketAddr, args: &[String]) {
        if args.len() != 2 {
            self.send_to(addr, "400 Bad Request");
            return;
        }

        let client_uuid = match self.get_client_uuid(addr) {
            Some(uuid) => uuid,
            None => {
                self.send_to(addr, "401 Unauthorized");
                return;
            }
        };

        let team_uuid = &args[1];
        if let Some(team) = self.db.teams.get_mut(team_uuid) {
            if !team.subscribers.iter().any(|u| u == &client_uuid) {
                self.send_to(addr, "401 Unauthorized: Not subscribed to team");
                return;
            }

            team.subscribers.retain(|u| u != &client_uuid);
            self.reset_client_context_if_inside_team(addr, team_uuid);

            ffi::call_user_unsubscribed(team_uuid, &client_uuid);
            self.send_to(addr, &format!("200 UNSUBSCRIBED|{client_uuid}|{team_uuid}"));
        } else {
            self.send_to(addr, "404 Not Found: Team not found");
        }
    }

    pub(crate) fn cmd_create(&mut self, addr: SocketAddr, args: &[String]) {
        let client = match self.clients.get(&addr) {
            Some(c) => c,
            None => return,
        };

        let client_uuid = match &client.uuid {
            Some(uuid) => uuid.clone(),
            None => {
                self.send_to(addr, "401 Unauthorized");
                return;
            }
        };

        let context = client.use_context.clone();

        if let Some(team_uuid) = Self::context_team_uuid(&context)
            && !self.is_subscribed_to_team(&client_uuid, team_uuid)
        {
            self.send_to(addr, "401 Unauthorized: Not subscribed to team");
            return;
        }

        match context {
            UseContext::Global => {
                if args.len() != 3 {
                    self.send_to(addr, "400 Bad Request");
                    return;
                }

                let name = &args[1];
                let desc = &args[2];

                if name.len() > MAX_NAME_LENGTH || desc.len() > MAX_DESCRIPTION_LENGTH {
                    self.send_to(addr, "400 Bad Request: Length error");
                    return;
                }

                if self.db.teams.values().any(|t| t.name == *name) {
                    self.send_to(addr, "409 Conflict: Team already exists");
                    return;
                }

                let new_uuid = generate_uuid();
                let team = Team {
                    uuid: new_uuid.clone(),
                    name: name.clone(),
                    description: desc.clone(),
                    subscribers: vec![client_uuid.clone()],
                    channels: HashMap::new(),
                };
                self.db.teams.insert(new_uuid.clone(), team);

                ffi::call_team_created(&new_uuid, name, &client_uuid);
                self.send_to(addr, &format!("200 TEAM_CREATED|{new_uuid}|{name}|{desc}"));
            }
            UseContext::Team(team_uuid) => {
                if args.len() != 3 {
                    self.send_to(addr, "400 Bad Request");
                    return;
                }

                let name = &args[1];
                let desc = &args[2];

                if name.len() > MAX_NAME_LENGTH || desc.len() > MAX_DESCRIPTION_LENGTH {
                    self.send_to(addr, "400 Bad Request: Length error");
                    return;
                }

                let team = match self.db.teams.get_mut(&team_uuid) {
                    Some(t) => t,
                    None => {
                        self.send_to(addr, "404 Not Found: Team unknown");
                        return;
                    }
                };

                if team.channels.values().any(|c| c.name == *name) {
                    self.send_to(addr, "409 Conflict: Channel already exists");
                    return;
                }

                let new_uuid = generate_uuid();
                let channel = Channel {
                    uuid: new_uuid.clone(),
                    name: name.clone(),
                    description: desc.clone(),
                    threads: HashMap::new(),
                };
                team.channels.insert(new_uuid.clone(), channel);

                ffi::call_channel_created(&team_uuid, &new_uuid, name);
                self.send_to(
                    addr,
                    &format!("200 CHANNEL_CREATED|{new_uuid}|{name}|{desc}"),
                );

                self.send_event_to_team_subscribers(
                    &team_uuid,
                    &format!("EVENT CHANNEL_CREATED|{new_uuid}|{name}|{desc}"),
                );
            }
            UseContext::Channel(team_uuid, channel_uuid) => {
                if args.len() != 3 {
                    self.send_to(addr, "400 Bad Request");
                    return;
                }

                let title = &args[1];
                let body = &args[2];

                if title.len() > MAX_NAME_LENGTH || body.len() > MAX_BODY_LENGTH {
                    self.send_to(addr, "400 Bad Request: Length error");
                    return;
                }

                let team = match self.db.teams.get_mut(&team_uuid) {
                    Some(t) => t,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Team");
                        return;
                    }
                };

                let channel = match team.channels.get_mut(&channel_uuid) {
                    Some(c) => c,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Channel");
                        return;
                    }
                };

                let new_uuid = generate_uuid();
                let timestamp = std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let thread = Thread {
                    uuid: new_uuid.clone(),
                    title: title.clone(),
                    message: body.clone(),
                    author_uuid: client_uuid.clone(),
                    replies: Vec::new(),
                    timestamp,
                };
                channel.threads.insert(new_uuid.clone(), thread);

                ffi::call_thread_created(&channel_uuid, &new_uuid, &client_uuid, title, body);
                self.send_to(
                    addr,
                    &format!(
                        "200 THREAD_CREATED|{new_uuid}|{client_uuid}|{timestamp}|{title}|{body}"
                    ),
                );

                self.send_event_to_team_subscribers(
                    &team_uuid,
                    &format!(
                        "EVENT THREAD_CREATED|{new_uuid}|{client_uuid}|{timestamp}|{title}|{body}"
                    ),
                );
            }
            UseContext::Thread(team_uuid, channel_uuid, thread_uuid) => {
                if args.len() != 2 {
                    self.send_to(addr, "400 Bad Request");
                    return;
                }

                let reply = &args[1];

                if reply.len() > MAX_BODY_LENGTH {
                    self.send_to(addr, "400 Bad Request: Reply too long");
                    return;
                }

                let team = match self.db.teams.get_mut(&team_uuid) {
                    Some(t) => t,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Team");
                        return;
                    }
                };

                let channel = match team.channels.get_mut(&channel_uuid) {
                    Some(c) => c,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Channel");
                        return;
                    }
                };

                let thread = match channel.threads.get_mut(&thread_uuid) {
                    Some(t) => t,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Thread");
                        return;
                    }
                };

                let timestamp = std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let message = Message {
                    sender_uuid: client_uuid.clone(),
                    receiver_uuid: thread_uuid.clone(),
                    body: reply.clone(),
                    timestamp,
                };
                thread.replies.push(message);

                ffi::call_reply_created(&thread_uuid, &client_uuid, reply);
                self.send_to(
                    addr,
                    &format!("200 REPLY_CREATED|{thread_uuid}|{client_uuid}|{timestamp}|{reply}"),
                );

                self.send_event_to_team_subscribers(
                    &team_uuid,
                    &format!(
                        "EVENT THREAD_REPLY_RECEIVED|{team_uuid}|{thread_uuid}|{client_uuid}|{reply}"
                    ),
                );
            }
        }
    }

    pub(crate) fn cmd_list(&mut self, addr: SocketAddr) {
        let client = match self.clients.get(&addr) {
            Some(c) => c,
            None => return,
        };

        let client_uuid = match &client.uuid {
            Some(uuid) => uuid.clone(),
            None => {
                self.send_to(addr, "401 Unauthorized: Please login first");
                return;
            }
        };

        let context = client.use_context.clone();

        if let Some(team_uuid) = Self::context_team_uuid(&context)
            && !self.is_subscribed_to_team(&client_uuid, team_uuid)
        {
            self.send_to(addr, "401 Unauthorized: Not subscribed to team");
            return;
        }

        match context {
            UseContext::Global => {
                let mut response = String::from("200 LIST_TEAMS");
                for team in self.db.teams.values() {
                    response.push_str(&format!(
                        "|{}:{}:{}",
                        team.uuid, team.name, team.description
                    ));
                }
                self.send_to(addr, &response);
            }
            UseContext::Team(team_uuid) => {
                let team = match self.db.teams.get(&team_uuid) {
                    Some(t) => t,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Team");
                        return;
                    }
                };

                let mut response = String::from("200 LIST_CHANNELS");
                for channel in team.channels.values() {
                    response.push_str(&format!(
                        "|{}:{}:{}",
                        channel.uuid, channel.name, channel.description
                    ));
                }
                self.send_to(addr, &response);
            }
            UseContext::Channel(team_uuid, channel_uuid) => {
                let team = match self.db.teams.get(&team_uuid) {
                    Some(t) => t,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Team");
                        return;
                    }
                };

                let channel = match team.channels.get(&channel_uuid) {
                    Some(c) => c,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Channel");
                        return;
                    }
                };

                let mut response = String::from("200 LIST_THREADS");
                for thread in channel.threads.values() {
                    response.push_str(&format!(
                        "|{}:{}:{}:{}:{}",
                        thread.uuid,
                        thread.author_uuid,
                        thread.timestamp,
                        thread.title,
                        thread.message
                    ));
                }
                self.send_to(addr, &response);
            }
            UseContext::Thread(team_uuid, channel_uuid, thread_uuid) => {
                let team = match self.db.teams.get(&team_uuid) {
                    Some(t) => t,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Team");
                        return;
                    }
                };

                let channel = match team.channels.get(&channel_uuid) {
                    Some(c) => c,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Channel");
                        return;
                    }
                };

                let thread = match channel.threads.get(&thread_uuid) {
                    Some(t) => t,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Thread");
                        return;
                    }
                };

                let mut response = String::from("200 LIST_REPLIES");
                for reply in &thread.replies {
                    response.push_str(&format!(
                        "|{}:{}:{}:{}",
                        thread.uuid, reply.sender_uuid, reply.timestamp, reply.body
                    ));
                }
                self.send_to(addr, &response);
            }
        }
    }

    pub(crate) fn cmd_info(&mut self, addr: SocketAddr) {
        let client = match self.clients.get(&addr) {
            Some(c) => c,
            None => return,
        };

        let client_uuid = match &client.uuid {
            Some(uuid) => uuid.clone(),
            None => {
                self.send_to(addr, "401 Unauthorized");
                return;
            }
        };

        let context = client.use_context.clone();

        if let Some(team_uuid) = Self::context_team_uuid(&context)
            && !self.is_subscribed_to_team(&client_uuid, team_uuid)
        {
            self.send_to(addr, "401 Unauthorized: Not subscribed to team");
            return;
        }

        match context {
            UseContext::Global => {
                let user = self.db.users.get(&client_uuid).unwrap();
                let status = if user.is_connected { "1" } else { "0" };
                self.send_to(
                    addr,
                    &format!("200 INFO_USER|{}|{}|{status}", user.uuid, user.name),
                );
            }
            UseContext::Team(team_uuid) => {
                let team = match self.db.teams.get(&team_uuid) {
                    Some(t) => t,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Team");
                        return;
                    }
                };

                self.send_to(
                    addr,
                    &format!("200 INFO_TEAM|{}|{}|{}", team.uuid, team.name, team.description),
                );
            }
            UseContext::Channel(team_uuid, channel_uuid) => {
                let team = match self.db.teams.get(&team_uuid) {
                    Some(t) => t,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Team");
                        return;
                    }
                };

                let channel = match team.channels.get(&channel_uuid) {
                    Some(c) => c,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Channel");
                        return;
                    }
                };

                self.send_to(
                    addr,
                    &format!(
                        "200 INFO_CHANNEL|{}|{}|{}",
                        channel.uuid, channel.name, channel.description
                    ),
                );
            }
            UseContext::Thread(team_uuid, channel_uuid, thread_uuid) => {
                let team = match self.db.teams.get(&team_uuid) {
                    Some(t) => t,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Team");
                        return;
                    }
                };

                let channel = match team.channels.get(&channel_uuid) {
                    Some(c) => c,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Channel");
                        return;
                    }
                };

                let thread = match channel.threads.get(&thread_uuid) {
                    Some(t) => t,
                    None => {
                        self.send_to(addr, "404 Not Found: Unknown Thread");
                        return;
                    }
                };

                self.send_to(
                    addr,
                    &format!(
                        "200 INFO_THREAD|{}|{}|{}|{}|{}",
                        thread.uuid,
                        thread.author_uuid,
                        thread.timestamp,
                        thread.title,
                        thread.message
                    ),
                );
            }
        }
    }

    pub(crate) fn cmd_subscribed(&mut self, addr: SocketAddr, args: &[String]) {
        let client_uuid = match self.get_client_uuid(addr) {
            Some(uuid) => uuid,
            None => {
                self.send_to(addr, "401 Unauthorized");
                return;
            }
        };

        if args.len() == 1 {
            let mut response = String::from("200 SUBSCRIBED_TEAMS");
            for team in self.db.teams.values() {
                if team.subscribers.iter().any(|u| u == &client_uuid) {
                    response.push_str(&format!(
                        "|{}:{}:{}",
                        team.uuid, team.name, team.description
                    ));
                }
            }
            self.send_to(addr, &response);
        } else if args.len() == 2 {
            let team_uuid = &args[1];
            let team = match self.db.teams.get(team_uuid) {
                Some(t) => t,
                None => {
                    self.send_to(addr, "404 Not Found: Unknown Team");
                    return;
                }
            };

            if !team.subscribers.iter().any(|u| u == &client_uuid) {
                self.send_to(addr, "401 Unauthorized: Not subscribed to team");
                return;
            }

            let mut response = String::from("200 SUBSCRIBED_USERS");
            for sub_uuid in &team.subscribers {
                if let Some(user) = self.db.users.get(sub_uuid) {
                    let status = if user.is_connected { "1" } else { "0" };
                    response.push_str(&format!("|{}:{}:{status}", user.uuid, user.name));
                }
            }
            self.send_to(addr, &response);
        } else {
            self.send_to(addr, "400 Bad Request");
        }
    }
}
