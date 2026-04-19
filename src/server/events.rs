use std::net::SocketAddr;

use crate::{client::UseContext, server::Server};

impl Server {
    pub fn is_subscribed_to_team(&self, user_uuid: &str, team_uuid: &str) -> bool {
        self.db
            .teams
            .get(team_uuid)
            .map(|team| team.subscribers.iter().any(|u| u == user_uuid))
            .unwrap_or(false)
    }

    pub fn context_team_uuid(context: &UseContext) -> Option<&str> {
        match context {
            UseContext::Global => None,
            UseContext::Team(team_uuid) => Some(team_uuid.as_str()),
            UseContext::Channel(team_uuid, _) => Some(team_uuid.as_str()),
            UseContext::Thread(team_uuid, _, _) => Some(team_uuid.as_str()),
        }
    }

    pub fn reset_client_context_if_inside_team(&mut self, addr: SocketAddr, team_uuid: &str) {
        if let Some(client) = self.clients.get_mut(&addr) {
            let must_reset = match &client.use_context {
                UseContext::Global => false,
                UseContext::Team(current_team) => current_team == team_uuid,
                UseContext::Channel(current_team, _) => current_team == team_uuid,
                UseContext::Thread(current_team, _, _) => current_team == team_uuid,
            };

            if must_reset {
                client.use_context = UseContext::Global;
            }
        }
    }

    pub fn send_event_to_user(&mut self, user_uuid: &str, msg: &str) {
        for client in self.clients.values_mut() {
            match client.uuid {
                Some(ref uuid) if uuid == user_uuid => {
                    client.queue_message(msg);
                    break;
                }
                _ => continue,
            }
        }
    }

    pub fn send_event_to_team_subscribers(&mut self, team_uuid: &str, msg: &str) {
        let subscribers = match self.db.teams.get(team_uuid) {
            Some(team) => team.subscribers.clone(),
            None => return,
        };

        for subscriber_uuid in subscribers {
            for client in self.clients.values_mut() {
                if let Some(ref uuid) = client.uuid
                    && uuid == &subscriber_uuid
                {
                    client.queue_message(msg);
                }
            }
        }
    }
}
