use std::sync::mpsc::{channel, Sender, Receiver};
use std::net::{UdpSocket, TcpStream, SocketAddr};
use laminar::{Socket, SocketEvent, Packet};
use crossbeam_channel as cb;
use crate::common;
use std::collections::HashMap;
use std::thread;
use anyhow::Result;
use std::str::FromStr;
use std::time::Duration;

pub fn start(port: u16, name: String, server: SocketAddr) -> Result<Sender<()>> {
    let mut socket = Socket::bind_any()?;

    let mut payload = vec![1];
    payload.extend_from_slice(&name.into_bytes());
    let packet = Packet::reliable_ordered(server, payload, None);

    socket.send(packet)?;
    
    let (stop_sender, stop_receiver) = channel();

    let sender = socket.get_packet_sender();
    let receiver = socket.get_event_receiver();

    thread::spawn(move || common::poll_socket(stop_receiver, socket));

    thread::spawn(move || handle_connections(port, server, sender, receiver));

    Ok(stop_sender)
}

fn handle_connections(
    port: u16,
    server: SocketAddr,
    sender: cb::Sender<Packet>,
    receiver: cb::Receiver<SocketEvent>)
{
    let mut clients: HashMap<SocketAddr, cb::Sender<Packet>> = HashMap::new();
    while let Ok(event) = receiver.recv() {

        match event {
            SocketEvent::Packet(packet) => {
                let addr = packet.addr();
                if addr == server {
                    if let Ok(client_addr) = String::from_utf8(packet.payload().to_vec()) {
                        if let Ok(client_addr) = SocketAddr::from_str(&client_addr) {
                            let (client_sender, client_receiver) = cb::unbounded();
                            clients.insert(client_addr, client_sender);

                            let initial_packet = Packet::reliable_ordered(client_addr, vec![0], None);
                            if sender.send(initial_packet).is_ok() {
                                let sender = sender.clone();
                                thread::spawn(move || handle_connection(sender, client_receiver, port, client_addr));
                            }
                        }
                    }
                } else {
                    if let Some(sender) = clients.get(&addr) {
                        if sender.send(packet).is_err() {
                            clients.remove(&addr);
                        }
                    }
                }
            },
            SocketEvent::Disconnect(addr) => {
                clients.remove(&addr);
            },
            _ => {}
        }
    }
}

fn handle_connection(
    sender: cb::Sender<Packet>,
    receiver: cb::Receiver<Packet>,
    port: u16,
    client_addr: SocketAddr)
{
    let (tcp_receiver, udp_receiver) = common::receivers(receiver);
    let (tcp_sender, udp_sender) = (sender.clone(), sender);

    thread::spawn(move || handle_tcp(port, tcp_sender, tcp_receiver, client_addr));
    thread::spawn(move || handle_udp(port, udp_sender, udp_receiver, client_addr));
}

fn handle_tcp(port: u16, sender: cb::Sender<Packet>, receiver: Receiver<Packet>, client_addr: SocketAddr) -> Result<()> {
    let tcp = TcpStream::connect(("127.0.0.1", port))?;

    thread::sleep(Duration::from_secs(2));
    drop(receiver.try_recv());

    common::handle_tcp(tcp, sender, receiver, client_addr)
}

fn handle_udp(port: u16, sender: cb::Sender<Packet>, receiver: Receiver<Packet>, client_addr: SocketAddr) -> Result<()> {
    let udp = UdpSocket::bind("0.0.0.0:0")?;
    udp.connect(("127.0.0.1", port))?;
    common::handle_udp(udp, sender, receiver, client_addr)
}
