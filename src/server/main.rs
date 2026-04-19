pub mod client;
pub mod commands;
pub mod events;
pub mod models;
pub mod server;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(flag) = args.get(1)
        && flag == "--help"
    {
        println!(
            "USAGE: ./myteams_server port\n\nport is the port number on which the server socket listens."
        );
        std::process::exit(0);
    }
    if args.len() != 2 {
        println!("USAGE: ./myteams_server port");
        std::process::exit(84);
    }

    let port: u16 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            println!("Invalid port number.");
            std::process::exit(84);
        }
    };

    my_teams::ffi::setup_signal_handler();

    match server::Server::new(port) {
        Ok(mut srv) => srv.run(),
        Err(e) => {
            println!("Error initialising server: {e}");
            std::process::exit(84);
        }
    }
}
