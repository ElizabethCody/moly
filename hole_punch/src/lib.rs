pub mod server;
pub mod client;
pub mod host;
mod common;

#[cfg(test)]
mod tests {
    use crate::{server, client, host};
    use std::net::{TcpListener, TcpStream, SocketAddr};
    use std::thread;
    use std::str::FromStr;
    use std::time::Duration;
    use std::io::{Read, Write};

    #[test]
    fn test() {
        // Test sending a string over a TCP connection that goes through the UDP
        // connection established by this library. This test runs entirely on one
        // machine, so it obviously follows that the actual hole punch is not tested.
        let server_addr = SocketAddr::from_str("127.0.0.1:1234").unwrap();
        let host_addr = SocketAddr::from_str("127.0.0.1:1235").unwrap();
        let client_addr = SocketAddr::from_str("127.0.0.1:1236").unwrap();

        let host = TcpListener::bind(host_addr).unwrap();

        thread::spawn(move || server::start(1234, 1));
        let host_stop = host::start(1235, String::from("test"), server_addr).unwrap();
        thread::sleep(Duration::from_secs(1));
        let client_stop = client::start(1236, String::from("test"), server_addr).unwrap();

        thread::sleep(Duration::from_secs(3));

        let sent_message = String::from("TEST MESSAGE");

        let mut client = TcpStream::connect(client_addr).unwrap();
        client.write(sent_message.as_bytes()).unwrap();


        let mut host = host.accept().unwrap().0;
        let mut buffer = vec![0; 12];
        host.read_exact(&mut buffer).unwrap();
        let received_message = String::from_utf8(buffer.to_vec()).unwrap();
        assert!(sent_message == received_message);
    }
}
