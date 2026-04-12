use crate::client::command::Command;
use crate::client::context::Context;

pub fn encode_command(cmd: &Command, ctx: &Context) -> String {
    match cmd {
        Command::Help => "HELP\n".to_string(),
        Command::Login { user_name } => format!("LOGIN|{}\n", user_name),
        Command::Logout => "LOGOUT\n".to_string(),
        Command::Users => "USERS\n".to_string(),
        Command::User { user_uuid } => format!("USER|{}\n", user_uuid),
        Command::Send { user_uuid, body } => format!("SEND|{}|{}\n", user_uuid, body),
        Command::Messages { user_uuid } => format!("MESSAGES|{}\n", user_uuid),
        Command::Subscribe { team_uuid } => format!("SUBSCRIBE|{}\n", team_uuid),
        Command::Subscribed { team_uuid } => match team_uuid {
            Some(id) => format!("SUBSCRIBED|{}\n", id),
            None => "SUBSCRIBED\n".to_string(),
        },
        Command::Unsubscribe { team_uuid } => format!("UNSUBSCRIBE|{}\n", team_uuid),
        Command::Use {
            team_uuid,
            channel_uuid,
            thread_uuid,
        } => format!(
            "USE|{}|{}|{}\n",
            team_uuid.clone().unwrap_or_default(),
            channel_uuid.clone().unwrap_or_default(),
            thread_uuid.clone().unwrap_or_default()
        ),
        Command::Create { args } => match (&ctx.team_uuid, &ctx.channel_uuid, &ctx.thread_uuid) {
            (None, None, None) => format!("CREATE_TEAM|{}|{}\n", args[0], args[1]),
            (Some(team), None, None) => format!("CREATE_CHANNEL|{}|{}|{}\n", team, args[0], args[1]),
            (Some(team), Some(channel), None) => {
                format!("CREATE_THREAD|{}|{}|{}|{}\n", team, channel, args[0], args[1])
            }
            (Some(team), Some(channel), Some(thread)) => {
                format!("CREATE_REPLY|{}|{}|{}|{}\n", team, channel, thread, args[0])
            }
            _ => "ERROR|INVALID_CONTEXT\n".to_string(),
        },
        Command::List => match (&ctx.team_uuid, &ctx.channel_uuid, &ctx.thread_uuid) {
            (None, None, None) => "LIST_TEAMS\n".to_string(),
            (Some(team), None, None) => format!("LIST_CHANNELS|{}\n", team),
            (Some(team), Some(channel), None) => format!("LIST_THREADS|{}|{}\n", team, channel),
            (Some(team), Some(channel), Some(thread)) => {
                format!("LIST_REPLIES|{}|{}|{}\n", team, channel, thread)
            }
            _ => "ERROR|INVALID_CONTEXT\n".to_string(),
        },
        Command::Info => match (&ctx.team_uuid, &ctx.channel_uuid, &ctx.thread_uuid) {
            (None, None, None) => "INFO_USER\n".to_string(),
            (Some(team), None, None) => format!("INFO_TEAM|{}\n", team),
            (Some(team), Some(channel), None) => format!("INFO_CHANNEL|{}|{}\n", team, channel),
            (Some(team), Some(channel), Some(thread)) => {
                format!("INFO_THREAD|{}|{}|{}\n", team, channel, thread)
            }
            _ => "ERROR|INVALID_CONTEXT\n".to_string(),
        },
    }
}
