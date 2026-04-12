use crate::client::command::Command;

fn tokenize_input(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.trim().chars().collect();
    let mut i = 0;

    //read loop
    while i < chars.len() {
        //skip spaces
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        //check if only spaces end loop
        if i >= chars.len() {
            break;
        }

        //look for quote for argument begining
        if chars[i] == '"' {
            i += 1;
            let start = i;

            //find closing quote
            while i < chars.len() && chars[i] != '"' {
                i += 1;
            }

            //if we reach end without quote
            if i >= chars.len() {
                return Err("Missing closing quote".to_string());
            }

            //extract the token
            let token: String = chars[start..i].iter().collect();
            tokens.push(token);
            i += 1;
        } else {
            //handle non quoted like commands
            let start = i;
            while i < chars.len() && !chars[i].is_whitespace() {
                i += 1;
            }
            let token: String = chars[start..i].iter().collect();
            tokens.push(token);
        }
    }

    Ok(tokens)
}

pub fn parse_command(input: &str) -> Result<Command, String> {
    let tokens = tokenize_input(input)?;

    if tokens.is_empty() {
        return Err("Empty command".to_string());
    }

    match tokens[0].as_str() {
        "/help" => {
            if tokens.len() != 1 {
                return Err("/help takes no arguments".to_string());
            }
            Ok(Command::Help)
        }
        "/login" => {
            if tokens.len() != 2 {
                return Err(r#"/login requires "user_name""#.to_string());
            }
            Ok(Command::Login {
                user_name: tokens[1].clone(),
            })
        }
        "/logout" => {
            if tokens.len() != 1 {
                return Err("/logout takes no arguments".to_string());
            }
            Ok(Command::Logout)
        }
        "/users" => {
            if tokens.len() != 1 {
                return Err("/users takes no arguments".to_string());
            }
            Ok(Command::Users)
        }
        "/user" => {
            if tokens.len() != 2 {
                return Err(r#"/user requires "user_uuid""#.to_string());
            }
            Ok(Command::User {
                user_uuid: tokens[1].clone(),
            })
        }
        "/send" => {
            if tokens.len() != 3 {
                return Err(r#"/send requires "user_uuid" "message_body""#.to_string());
            }
            Ok(Command::Send {
                user_uuid: tokens[1].clone(),
                body: tokens[2].clone(),
            })
        }
        "/messages" => {
            if tokens.len() != 2 {
                return Err(r#"/messages requires "user_uuid""#.to_string());
            }
            Ok(Command::Messages {
                user_uuid: tokens[1].clone(),
            })
        }
        "/subscribe" => {
            if tokens.len() != 2 {
                return Err(r#"/subscribe requires "team_uuid""#.to_string());
            }
            Ok(Command::Subscribe {
                team_uuid: tokens[1].clone(),
            })
        }
        "/subscribed" => {
            if tokens.len() > 2 {
                return Err(r#"/subscribed accepts zero or one "team_uuid""#.to_string());
            }
            Ok(Command::Subscribed {
                team_uuid: tokens.get(1).cloned(),
            })
        }
        "/unsubscribe" => {
            if tokens.len() != 2 {
                return Err(r#"/unsubscribe requires "team_uuid""#.to_string());
            }
            Ok(Command::Unsubscribe {
                team_uuid: tokens[1].clone(),
            })
        }
        "/use" => {
            if tokens.len() > 4 {
                return Err(r#"/use accepts up to "team_uuid" "channel_uuid" "thread_uuid""#.to_string());
            }
            Ok(Command::Use {
                team_uuid: tokens.get(1).cloned(),
                channel_uuid: tokens.get(2).cloned(),
                thread_uuid: tokens.get(3).cloned(),
            })
        }
        "/create" => Ok(Command::Create {
            args: tokens[1..].to_vec(),
        }),
        "/list" => {
            if tokens.len() != 1 {
                return Err("/list takes no arguments".to_string());
            }
            Ok(Command::List)
        }
        "/info" => {
            if tokens.len() != 1 {
                return Err("/info takes no arguments".to_string());
            }
            Ok(Command::Info)
        }
        _ => Err("Unknown command".to_string()),
    }
}
