#[macro_use]
use std::collections::HashMap;
use byteorder::{ByteOrder, NetworkEndian};
use reliudp::RUdpServer;
use std::net::TcpListener;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use types::*;

pub struct MarketData {
    // ips: Vec<String>,
    ip_addr: &'static str,
    port: u32,
    price_recv: Receiver<PriceInfo>,
    symb_to_prices: HashMap<String, PriceInfo>,
}

impl MarketData {
    pub fn new(ip_addr: &'static str, port: u32, price_recv: Receiver<PriceInfo>) -> Self {
        MarketData {
            // ips: Vec::new(),
            ip_addr: ip_addr,
            port: port,
            price_recv: price_recv,
            symb_to_prices: HashMap::new(),
        }
    }

    // pub fn add_subscriber(&mut self, &str ip) {
    //     self.ips.push(ip);
    // }

    pub fn get_symb_to_prices(&self) -> &HashMap<String, PriceInfo> {
        return &self.symb_to_prices;
    }

    pub fn update_price(&mut self, new_price_info: PriceInfo) {
        self.symb_to_prices.insert(
            new_price_info.get_symbol().ticker().to_string(),
            new_price_info,
        );
    }

    pub fn run(&self) -> () {
        // let port = get_available_port().expect("[ERROR] no available port in range 8000-62000");
        let mut server =
            RUdpServer::new(format!("{}:{}", self.ip_addr, self.port))
                .expect("[ERROR] failed to create server");

        println!("[INFO] started UDP server on {}:{}", self.ip_addr, self.port);
        loop {
            server.next_tick().expect("[ERROR] udp server failed on next tick");
            for server_event in server.drain_events() {
                println!("Server: Incoming event {:?}", server_event);
            }

            // ticker + bid + bid size + ask + ask size
            let curr_prices: &HashMap<String, PriceInfo> = self.get_symb_to_prices();
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
            let data_wrapped: Arc<[u8]> = Arc::from(data.into_boxed_slice());

            server.send_data(&data_wrapped, reliudp::MessageType::KeyMessage);
            ::std::thread::sleep(::std::time::Duration::from_millis(5));
        }
    }
}

fn port_is_available(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn get_available_port() -> Option<u16> {
    (8000..62000).find(|port| port_is_available(*port))
}
