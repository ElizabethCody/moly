use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use std::thread;
use laminar::{Socket, Packet, DeliveryGuarantee};
use crossbeam_channel as cb;
use anyhow::Result;
use std::net::{SocketAddr, UdpSocket, TcpStream};
use std::io::{Read, Write};

pub fn poll_socket(stop_receiver: Receiver<()>, mut socket: Socket) {
    let duration = Duration::from_millis(1);
    while stop_receiver.try_recv().is_err() {
        socket.manual_poll(Instant::now());
        thread::sleep(duration);
    }
}

pub fn receivers(receiver: cb::Receiver<Packet>) -> (Receiver<Packet>, Receiver<Packet>) {
    let (tcp_sender, tcp_receiver) = channel();
    let (udp_sender, udp_receiver) = channel();
    thread::spawn(move || {
        while let Ok(packet) = receiver.recv() {
            match packet.delivery_guarantee() {
                DeliveryGuarantee::Reliable => tcp_sender.send(packet).unwrap(),
                DeliveryGuarantee::Unreliable => udp_sender.send(packet).unwrap(),
            };
        }
    });
    (tcp_receiver, udp_receiver)
}

pub fn handle_tcp(tcp: TcpStream, sender: cb::Sender<Packet>, receiver: Receiver<Packet>, addr: SocketAddr) -> Result<()> {
    let mut tcp_send = tcp.try_clone()?;
    let mut tcp_recv = tcp;

    thread::spawn(move || {
        let mut buffer = vec![0; 10];
        while let Ok(bytes_read) = tcp_recv.read(&mut buffer) {
            sender.send(Packet::reliable_ordered(addr, buffer[..bytes_read].to_vec(), None))
                .unwrap();
        }
    });

    while let Ok(packet) = receiver.recv() {
        tcp_send.write(&packet.payload())?;
    }

    Ok(())
}

pub fn handle_udp(udp: UdpSocket, sender: cb::Sender<Packet>, receiver: Receiver<Packet>, addr: SocketAddr) -> Result<()> {
    let udp_send = udp.try_clone()?;
    let udp_recv = udp;

    thread::spawn(move || {
        let mut buffer = vec![0; 10];
        while let Ok(bytes_read) = udp_recv.recv(&mut buffer) {
            sender.send(Packet::unreliable(addr, buffer[..bytes_read].to_vec()))
                .unwrap();
        }
    });

    while let Ok(packet) = receiver.recv() {
        udp_send.send(&packet.payload())?;
    }

    Ok(())
}
