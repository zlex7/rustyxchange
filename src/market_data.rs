#[macro_use]

use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::LinkedList;
use std::sync::mpsc::{Sender, Receiver};
use std::cmp;
use std::sync::Arc;
use std::net::{TcpListener, TcpStream, Shutdown};
use reliudp::RUdpServer;
use byteorder::{NetworkEndian, ByteOrder};

use types::*;

lazy_static! {
    static ref MARKET_DATA_SERVER_HOST : String = "0.0.0.0".to_string();
    static ref MARKET_DATA_SERVER_PORT : String = "61244".to_string();
}

fn port_is_available(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn get_available_port() -> Option<u16> {
    (8000..62000)
        .find(|port| port_is_available(*port))
}

pub fn start_market_data_server(recv_price: Receiver<PriceInfo>) -> () {
    let mut provider = MarketDataProvider::new();
    let port = get_available_port().expect("not a single port from 8000-62000 is open???");
    let mut server = reliudp::RUdpServer::new(format!("{}:{}",MARKET_DATA_SERVER_HOST.to_string(),port)).expect("Failed to create server");
    println!("[INFO]: UDP server on {}:{}", MARKET_DATA_SERVER_HOST.to_string(), port);
    loop {
        server.next_tick();
        for server_event in server.drain_events() {
            println!("Server: Incoming event {:?}", server_event);
        }
        // ticker + bid + bid size + ask + ask size
        let curr_prices : &HashMap<String, PriceInfo> = provider.get_symb_to_prices();
        let mut data: Vec<u8> = vec![];
        // let mut data = [0; curr_prices.len()*(4 + 8 + 8 + 8 + 8)];
        for (ticker, price_info) in curr_prices.iter() {
            // set top-level market data
            let ticker_bytes = ticker.as_bytes();
            let num = NetworkEndian::read_u32(ticker_bytes);
            NetworkEndian::write_u32(&mut data, num);
            NetworkEndian::write_u64(&mut data, price_info.best_bid);
            NetworkEndian::write_u64(&mut data, price_info.bid_size);
            NetworkEndian::write_u64(&mut data, price_info.best_ask);
            NetworkEndian::write_u64(&mut data, price_info.ask_size);
        }
        let data_wrapped : Arc<[u8]> = Arc::from(data.into_boxed_slice());

        server.send_data(&data_wrapped, reliudp::MessageType::KeyMessage);
        ::std::thread::sleep(::std::time::Duration::from_millis(5));
    }
}

fn refresh_market_data(mut provider: MarketDataProvider, recv_price: &mut Receiver<PriceInfo>) -> () {
    loop {
        for i in 1..100 {
            if (recv_price.try_recv().is_err()){
                break;
            }
            let new_price_info = recv_price.recv().expect("no data in price info queue");
            provider.update_price(new_price_info);
        }
        // while (!recv_subscribe.try_recv.is_err()) {
        //     let new_sub_info = recv_subscribe.recv().expect("no data in subscribe info queue");
        //     provider.add_subscriber(new_sub_info.ip);
        // }
    }
}



