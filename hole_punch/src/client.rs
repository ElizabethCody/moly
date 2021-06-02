use std::sync::mpsc::{channel, Sender, Receiver};
use anyhow::Result;
use std::net::{TcpListener, UdpSocket, SocketAddr};
use laminar::{Socket, SocketEvent, Packet};
use std::thread;
use std::time::Duration;
use std::str::FromStr;
use crate::common;
use crossbeam_channel as cb;

pub fn start(port: u16, host: String, server: SocketAddr) -> Result<Sender<()>> {
    let tcp = TcpListener::bind(("127.0.0.1", port))?;
    let udp = UdpSocket::bind(("127.0.0.1", port))?;

    let socket = Socket::bind_any()?;

    let sender = socket.get_packet_sender();
    let receiver = socket.get_event_receiver();

    let (stop_sender, stop_receiver) = channel();
    thread::spawn(move || common::poll_socket(stop_receiver, socket));

    let mut payload = vec![0];
    payload.extend_from_slice(&host.into_bytes());
    let packet = Packet::reliable_ordered(server, payload, None);

    sender.send(packet)?;

    thread::spawn(move || {
        receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        match receiver.recv().unwrap() {
            SocketEvent::Packet(packet) => {
                let host_addr = String::from_utf8(packet.payload().to_vec()).unwrap();
                handle_connection(host_addr, sender, receiver, tcp, udp).unwrap();
            },
            _ => {}
        }
    });

    Ok(stop_sender)
}

fn handle_connection(
    host_addr: String,
    sender: cb::Sender<Packet>,
    receiver: cb::Receiver<SocketEvent>,
    tcp: TcpListener,
    udp: UdpSocket) -> Result<()> 
{
    let host_addr = SocketAddr::from_str(&host_addr)?;
    let initial_packet = Packet::reliable_ordered(host_addr, vec![0], None);

    sender.send(initial_packet)?;

    let (relay_sender, relay_receiver) = cb::unbounded();
    thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            match event {
                SocketEvent::Packet(packet) => relay_sender.send(packet).unwrap(),
                _ => {
                    println!("{:?}", &event);
                }
            }
        }
    });

    let (tcp_receiver, udp_receiver) = common::receivers(relay_receiver);
    let (tcp_sender, udp_sender) = (sender.clone(), sender);

    thread::spawn(move || handle_tcp(tcp, tcp_sender, tcp_receiver, host_addr));
    thread::spawn(move || handle_udp(udp, udp_sender, udp_receiver, host_addr));

    Ok(())
}

fn handle_tcp(tcp: TcpListener, sender: cb::Sender<Packet>, receiver: Receiver<Packet>, host_addr: SocketAddr) -> Result<()> {
    let tcp = tcp.accept()?.0;

    thread::sleep(Duration::from_secs(2));
    drop(receiver.try_recv());

    common::handle_tcp(tcp, sender, receiver, host_addr)
}

fn handle_udp(udp: UdpSocket, sender: cb::Sender<Packet>, receiver: Receiver<Packet>, host_addr: SocketAddr) -> Result<()> {
    let addr = udp.peek_from(&mut [0; 1])?.1;
    udp.connect(addr)?;
    common::handle_udp(udp, sender, receiver, host_addr)
}
