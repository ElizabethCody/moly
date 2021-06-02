use hole_punch;
use std::env::args;
use std::thread;
use std::net::SocketAddr;
use std::str::FromStr;

mod config;
mod gui;

fn main() {
    let args: Vec<String> = args().collect();

    if args.len() >= 2 {
        match args[1].as_str() {
            "server" => server(&args[2..]),
            "client" => client(&args[2..]),
            "host" => host(&args[2..]),
            _ => {}
        }
        return;
    }

    gui::run();
}

fn server(args: &[String]) {
    let port = args[0].parse()
        .expect("Parsing server port");
    let max_hosts = args[1].parse()
        .expect("Parsing maximum connected hosts");
    hole_punch::server::start(port, max_hosts);
}

fn client(args: &[String]) {
    let port = args[0].parse()
        .expect("Parsing client port");
    let host_name = args[1].to_owned();
    let server_addr = SocketAddr::from_str(&args[2])
        .expect("Parsing server address");
    hole_punch::client::start(port, host_name, server_addr)
        .expect("Starting client");
    thread::park();
}

fn host(args: &[String]) {
    let port = args[0].parse()
        .expect("Parsing host port");
    let host_name = args[1].to_owned();
    let server_addr = SocketAddr::from_str(&args[2])
        .expect("Parsing server address");
    hole_punch::host::start(port, host_name, server_addr)
        .expect("Starting host");
    thread::park();
}
