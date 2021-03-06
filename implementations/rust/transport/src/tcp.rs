use futures::io::Error;
#[allow(unused)]
use ockam_message::message::*;
use ockam_system::commands::RouterCommand::ReceiveMessage;
use ockam_system::commands::{OckamCommand, RouterCommand, TransportCommand};
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::net::{SocketAddr, TcpListener};
use std::time::Duration;

pub struct TcpManager {
    rx: std::sync::mpsc::Receiver<OckamCommand>,
    _tx: std::sync::mpsc::Sender<OckamCommand>,
    router_tx: std::sync::mpsc::Sender<OckamCommand>,
    _timeout: Duration,
    listener: Option<TcpListener>,
    connections: HashMap<String, TcpTransport>,
    addresses: Vec<String>,
}

impl TcpManager {
    pub fn connect(&mut self, address: SocketAddr) -> Result<Address, String> {
        let stream = TcpStream::connect(address);
        match stream {
            Ok(stream) => {
                stream.set_nonblocking(true).unwrap();
                stream.set_nodelay(true).unwrap();
                self.add_connection(stream);
                Ok(Address::TcpAddress(address))
            }
            Err(e) => Err(format!("tcp failed to connect: {}", e)),
        }
    }

    pub fn new(
        rx: std::sync::mpsc::Receiver<OckamCommand>,
        tx: std::sync::mpsc::Sender<OckamCommand>,
        router_tx: std::sync::mpsc::Sender<OckamCommand>,
        listen_addr: Option<SocketAddr>,
        tmo: Option<Duration>,
    ) -> Result<TcpManager, String> {
        router_tx
            .send(OckamCommand::Router(RouterCommand::Register(
                AddressType::Tcp,
                tx.clone(),
            )))
            .unwrap();
        let connections = HashMap::new();

        let timeout = tmo.unwrap_or(Duration::new(5, 0));

        return match listen_addr {
            Some(la) => {
                if let Ok(l) = TcpListener::bind(la) {
                    l.set_nonblocking(true).unwrap();
                    Ok(TcpManager {
                        rx,
                        _tx: tx,
                        router_tx,
                        _timeout: timeout,
                        listener: Some(l),
                        connections,
                        addresses: vec![],
                    })
                } else {
                    Err("failed to bind tcp listener".into())
                }
            }
            _ => Ok(TcpManager {
                rx,
                _tx: tx,
                router_tx,
                _timeout: timeout,
                listener: None,
                connections,
                addresses: vec![],
            }),
        };
    }

    fn add_connection(&mut self, stream: TcpStream) -> bool {
        stream.set_nonblocking(true).unwrap();
        stream.set_nodelay(true).unwrap();
        let peer_addr = stream.peer_addr().unwrap().clone();
        let tcp_xport = TcpTransport::new(stream, self.router_tx.clone()).unwrap();
        self.connections.insert(peer_addr.to_string(), tcp_xport);
        self.addresses.push(peer_addr.to_string());
        true
    }

    pub fn poll(&mut self) -> bool {
        let mut got: bool = true;
        let mut keep_going = true;

        while got && keep_going {
            // listen for connect
            got = false;
            if let Some(listener) = &self.listener {
                for s in listener.incoming() {
                    match s {
                        Ok(stream) => {
                            keep_going = self.add_connection(stream);
                            break;
                        }
                        Err(e) => match e.kind() {
                            io::ErrorKind::WouldBlock => {
                                break;
                            }
                            _ => {
                                println!("tcp listen error");
                                keep_going = false;
                                break;
                            }
                        },
                    }
                }
            }

            if let Ok(tc) = self.rx.try_recv() {
                match tc {
                    OckamCommand::Transport(TransportCommand::SendMessage(mut m)) => {
                        let addr = m.onward_route.addresses.get_mut(0).unwrap();
                        let addr = addr.address.as_string();
                        if let Some(tcp_xport) = self.connections.get_mut(&addr) {
                            match tcp_xport.send_message(m) {
                                Err(e) => {
                                    println!("send_message failed: {}", e);
                                    keep_going = false;
                                }
                                _ => {}
                            }
                        } else {
                            println!("can't find connection {}", addr);
                            println!("{} connections in hashmap", self.connections.len());
                            for (c, t) in &self.connections {
                                println!("{}", c);
                            }
                        }
                    }
                    OckamCommand::Transport(TransportCommand::Stop) => {
                        keep_going = false;
                        break;
                    }
                    _ => {
                        println!("unrecognized command");
                    }
                }
            } // end match rx.try_recv()

            // check for receives
            for a in &self.addresses {
                match self.connections.get_mut(&a.to_string()) {
                    Some(t) => {
                        if let Err(s) = t.receive_message() {
                            println!("tcp receive failed {}", s);
                            return false;
                        }
                    }
                    None => {}
                }
            }
        }

        keep_going
    }
}

pub struct TcpTransport {
    stream: TcpStream,
    router_tx: std::sync::mpsc::Sender<OckamCommand>,
}

impl TcpTransport {
    pub fn new(
        stream: TcpStream,
        router_tx: std::sync::mpsc::Sender<OckamCommand>,
    ) -> Result<TcpTransport, String> {
        let local_address = Address::TcpAddress(stream.local_addr().unwrap());
        Ok(TcpTransport { stream, router_tx })
    }

    pub fn send_message(&mut self, mut m: Message) -> Result<(), String> {
        m.onward_route.addresses.remove(0);
        let local_address = Address::TcpAddress(self.stream.local_addr().unwrap());
        m.return_route
            .addresses
            .insert(0, RouterAddress::from_address(local_address).unwrap());
        let mut v = vec![];
        Message::encode(&m, &mut v)?;
        return match self.stream.write(v.as_slice()) {
            Ok(_) => Ok(()),
            Err(_) => Err("tcp write failed".into()),
        };
    }

    pub fn receive_message(&mut self) -> Result<bool, String> {
        let mut buff = [0u8; 16348];
        self.stream.set_nonblocking(true);
        self.stream.set_nodelay(true);
        match self.stream.read(&mut buff) {
            Ok(len) => {
                if len == 0 {
                    return Ok(true);
                }
                return match Message::decode(&buff[0..len]) {
                    Ok((mut m, _)) => {
                        // fix up return tcp address with nat-ed address
                        let tcp_return = Address::TcpAddress(self.stream.peer_addr().unwrap());
                        m.return_route.addresses[0] =
                            RouterAddress::from_address(tcp_return).unwrap();
                        if !m.onward_route.addresses.is_empty()
                            && ((m.onward_route.addresses[0].a_type == AddressType::Udp)
                                || (m.onward_route.addresses[0].a_type == AddressType::Tcp))
                        {
                            match self.send_message(m) {
                                Err(s) => Err(s),
                                Ok(()) => Ok(true),
                            }
                        } else {
                            match self.router_tx.send(OckamCommand::Router(ReceiveMessage(m))) {
                                Ok(_unused) => Ok(true),
                                Err(_) => Err("send to router failed".to_string()),
                            }
                        }
                    }
                    _ => Err("decode failed".to_string()),
                };
            }
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => Ok(false),
                _ => Err("***tcp receive failed".to_string()),
            },
        }
    }
}
