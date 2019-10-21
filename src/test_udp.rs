
#[macro_use]
extern crate lazy_static;
extern crate getset;
extern crate csv;
extern crate byteorder;
extern crate reliudp;

use std::{str, u32, thread, fs};
use std::io::{BufReader, BufWriter, Read, Write};
use std::collections::{HashMap,HashSet};
use std::convert::TryInto;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::sync::mpsc::{Sender, Receiver, channel};
use reliudp::SocketEvent;
use byteorder::{NetworkEndian, ByteOrder};

mod types;
use types::*;

mod market_data;
use market_data::start_market_data_server;

fn main() {
    println!("starting main method...");
    let (market_data_sender, market_data_receiver): (Sender<PriceInfo>, Receiver<PriceInfo>) = channel();

    let mut symbols = HashSet::new();
    symbols.insert(Symbol::new("GOOG".to_string()));
    symbols.insert(Symbol::new("AAPL".to_string()));

    thread::spawn(|| {
        start_market_data_server(symbols, market_data_receiver);
    });
    
    thread::spawn(|| {
        let mut client = reliudp::RUdpSocket::connect("127.0.0.1:4567").expect("Failed to create client");
        for i in 0.. {
            client.next_tick().unwrap();
            for client_event in client.drain_events() {
                if let SocketEvent::Data(d) = client_event {
                    println!("Client: Incoming {:?} bytes (n={:?}) at frame {:?}, values = {:?}", d.len(), d[0], i, d);
                } else {
                    println!("Client: Incoming event {:?} at frame {:?}", client_event, i);
                }
            }
    
            ::std::thread::sleep(::std::time::Duration::from_millis(1));
        }
    });

    // ::std::thread::sleep(::std::time::Duration::from_millis(10));
    market_data_sender.send(PriceInfo::new(Symbol::new("GOOG".to_string()),10,1000,20,2000));
    // ::std::thread::sleep(::std::time::Duration::from_millis(10));
    market_data_sender.send(PriceInfo::new(Symbol::new("AAPL".to_string()),123,500,456,1500));
    
    loop {}
}