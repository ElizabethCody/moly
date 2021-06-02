use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use laminar::{Socket, Packet, SocketEvent};
use std::net::SocketAddr;
use crossbeam_channel::Sender;
use std::thread;
use anyhow::Result;
use std::io::{Error, ErrorKind};

/// Start server at the specified port on localhost. This function blocks forever.
///
/// # Arguments
/// * `port` - The port to which the server will bind and at which it will be
///     reachable by peers.
/// * `max_hosts` - The maximum number of hosts about which the server will store
///     connection information (name & address).
pub fn start(port: u16, max_hosts: usize) {
    let mut socket = Socket::bind(("127.0.0.1", port)).unwrap();
    
    let sender = socket.get_packet_sender();
    let receiver = socket.get_event_receiver();

    thread::spawn(move || socket.start_polling());

    // Hosts will register themselves with a name. Clients will use this name
    // to request the address of the associated host to avoid using addresses
    // directly.
    let hosts: Hosts = Arc::new(Mutex::new((HashMap::new(), HashMap::new())));

    // Wait for a connection, if it is incoming data, handle it. If a peer
    // disconnected, remove its information from `hosts` if it is a host.
    loop {
        if let Ok(event) = receiver.recv() {
            match event {
                SocketEvent::Packet(packet) => {
                    let sender = sender.clone();
                    let hosts = hosts.clone();
                    thread::spawn(move || handle_connection(packet, sender, hosts, max_hosts));
                },
                SocketEvent::Disconnect(addr) => {
                    let mut hosts = hosts.lock().unwrap();
                    if let Some(name) = hosts.1.get(&addr) {
                        let name = name.to_owned();
                        hosts.0.remove(&name);
                        hosts.1.remove(&addr);
                    }
                }
                _ => {}
            }
        }
    }
}

type Hosts = Arc<Mutex<(HashMap<String, SocketAddr>, HashMap<SocketAddr, String>)>>;

fn handle_connection(
    packet: Packet,
    sender: Sender<Packet>,
    hosts: Hosts,
    max_hosts: usize) -> Result<()>
{
    // Peers submit one packet to the server: the first byte of its payload
    // represents the type of connection (0 = Client, 1 = Host) and the rest
    // is the name at which the peer will register (if peer is a Host) or the
    // name of the Host to which it wants to connect (if peer is a Client).
    let payload = packet.payload();
    if payload.len() < 2 {
        let error = Error::new(ErrorKind::InvalidData, "");
        return Err(anyhow::Error::new(error));
    }
    let mode = payload[0];
    let name = String::from_utf8(payload[1..].to_vec())?;
    let addr = packet.addr();
    if mode == 0 {
        client(addr, sender, name, hosts);
    } else if mode == 1 {
        host(addr, name, hosts, max_hosts);
    }
    Ok(())
}

// Client submits the name of the host it would like to reach. Server exchanges
// the addresses of both peers with each other so that they can complete a peer-
// to-peer connection.
fn client(client_addr: SocketAddr, sender: Sender<Packet>, name: String, hosts: Hosts) {
    let hosts = hosts.lock().unwrap();
    if let Some(host_addr) = hosts.0.get(&name) {
        let host_addr_bytes = format!("{}", host_addr).into_bytes();
        let client_addr_bytes = format!("{}", &client_addr).into_bytes();

        let host_packet = Packet::reliable_ordered(*host_addr, client_addr_bytes, None);
        let client_packet = Packet::reliable_ordered(client_addr, host_addr_bytes, None);

        sender.send(host_packet).unwrap();
        sender.send(client_packet).unwrap();
    }
}

// Host submits the name at which it can be reached
fn host(addr: SocketAddr, name: String, hosts: Hosts, max_hosts: usize) {
    let mut hosts = hosts.lock().unwrap();
    if hosts.0.len() < max_hosts && !hosts.0.contains_key(&name) {
        hosts.0.insert(name, addr);
    }
}
