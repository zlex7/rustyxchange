use std::collections::HashMap;
use byteorder::{ByteOrder, NetworkEndian, WriteBytesExt};
use reliudp::RUdpServer;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use super::SYMBOLS;
use types::*;

pub struct MarketDataProvider {
    symb_to_prices: HashMap<String, PriceInfo>,
    ip_addr: &'static str,
    port: u32,
    receiver: Receiver<PriceInfo>,
}

impl MarketDataProvider {
    pub fn new(ip_addr: &'static str, port: u32, receiver: Receiver<PriceInfo>) -> MarketDataProvider {
        let mut symb_to_prices: HashMap<String, PriceInfo> = HashMap::new();
        for (ticker,symbol) in SYMBOLS.iter() {
            symb_to_prices.insert(ticker.to_string(), PriceInfo::new(symbol, 0, 0, 0, 0));
        }

        MarketDataProvider {
            symb_to_prices: symb_to_prices,
            ip_addr: ip_addr,
            port: port,
            receiver: receiver,
        }
    }

    pub fn update_price(&mut self, price_info: PriceInfo) {
        let ticker = price_info.get_symbol().ticker();
        self.symb_to_prices
            .insert(ticker.to_string(), price_info);
    }

    pub fn run(&mut self) {
        // let port = get_available_port().expect("not a single port from 8000-62000 is open???");
        let mut server = RUdpServer::new(format!("{}:{}", self.ip_addr, self.port))
            .expect("[ERROR] failed to create UDP server");
        println!("[INFO]: UDP server on {}:{}", self.ip_addr, self.port);

        loop {
            server.next_tick().unwrap();
            for server_event in server.drain_events() {
                println!("Server: Incoming event {:?}", server_event);
            }

            self.refresh();

            // ticker + bid + bid size + ask + ask size

            // let mut data : [u8] = [0; 36*curr_prices.len()];
            let mut data: Vec<u8> = vec![0; 36 * self.symb_to_prices.len()];
            // let mut data = [0; curr_prices.len()*(4 + 8 + 8 + 8 + 8)];
            for (ticker, price_info) in self.symb_to_prices.iter() {
                // println!("sending {:?} for {}", price_info, ticker);
                // set top-level market data
                let ticker_bytes = ticker.as_bytes();
                let num: u32 = NetworkEndian::read_u32(ticker_bytes);
                data.write_u32::<NetworkEndian>(num).unwrap();
                data.write_u64::<NetworkEndian>(price_info.best_bid)
                    .unwrap();
                data.write_u64::<NetworkEndian>(price_info.bid_size)
                    .unwrap();
                data.write_u64::<NetworkEndian>(price_info.best_ask)
                    .unwrap();
                data.write_u64::<NetworkEndian>(price_info.ask_size)
                    .unwrap();

                // NetworkEndian::write_u32(&mut data, num);
                // NetworkEndian::write_u64(&mut data, price_info.best_bid);
                // NetworkEndian::write_u64(&mut data, price_info.bid_size);
                // NetworkEndian::write_u64(&mut data, price_info.best_ask);
                // NetworkEndian::write_u64(&mut data, price_info.ask_size);
            }

            // let data_wrapped : Arc<[u8]> = Arc::from(data.iter().cloned().map(|x| x as u8).collect::<Vec<u8>>().into_boxed_slice());
            // println!("{:?}", data);
            let data_wrapped: Arc<[u8]> = Arc::from(data.into_boxed_slice());

            server.send_data(&data_wrapped, reliudp::MessageType::KeyMessage);
            ::std::thread::sleep(::std::time::Duration::from_millis(1));
        }
    }

    fn refresh(&mut self) {
        for _ in 0..100 {
            if let Ok(new_price_info) = self.receiver.try_recv() {
                println!("new price info: {:?}", new_price_info);
                self.update_price(new_price_info);
            }
        }
    }

    /*
    pub fn add_subscriber(&mut self, &str ip) {
        self.ips.push(ip);
    }

    pub fn get_symbol(&self) -> &Symbol {
        &self.symbol
    }

    pub fn get_symb_to_prices(&self) -> &HashMap<String, PriceInfo> {
        return &self.symb_to_prices;
    }
    */
}

// TODO: unit tests to make sure functions are working correctly
#[cfg(test)]
mod tests {
    use super::*;
    use reliudp::SocketEvent;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::{fs, str, thread, u32};

    #[test]
    fn test_connect() {
        // let (market_data_sender, market_data_receiver): (Sender<PriceInfo>, Receiver<PriceInfo>) = channel();
        // thread::spawn(|| {
        //     start_market_data_server(market_data_receiver);
        // });
        // thread::spawn(|| {
        //     let mut client = reliudp::RUdpSocket::connect("127.0.0.1:4567").expect("Failed to create client");
        //     for i in 0.. {
        //         client.next_tick().unwrap();
        //         for client_event in client.drain_events() {
        //             if let SocketEvent::Data(d) = client_event {
        //                 println!("Client: Incoming {:?} bytes (n={:?}) at frame {:?}", d.len(), d[0], i);
        //             } else {
        //                 println!("Client: Incoming event {:?} at frame {:?}", client_event, i);
        //             }
        //         }
        //         ::std::thread::sleep(::std::time::Duration::from_millis(1));
        //     }
        // });

        // market_data_sender.send(PriceInfo::new(Symbol::new("GOOG".to_string()),10,1000,20,2000));

        // market_data_sender.send(PriceInfo::new(Symbol::new("AAPL".to_string()),123,500,456,1500));
    }
}

// fn refresh_market_data(mut provider: MarketDataProvider, recv_price: &mut Receiver<PriceInfo>) -> () {
//     loop {
//         for i in 1..100 {
//             if (recv_price.try_recv().is_err()){
//                 break;
//             }
//             let new_price_info = recv_price.recv().expect("no data in price info queue");
//             provider.update_price(new_price_info);
//         }
//         // while (!recv_subscribe.try_recv.is_err()) {
//         //     let new_sub_info = recv_subscribe.recv().expect("no data in subscribe info queue");
//         //     provider.add_subscriber(new_sub_info.ip);
//         // }
//     }
// }

/*
pub fn start_market_data_server(recv_price: Receiver<PriceInfo>) -> () {
    let mut provider = MarketDataProvider::new();
    let port = 4567;
    // let port = get_available_port().expect("not a single port from 8000-62000 is open???");
    let mut server = RUdpServer::new(format!("{}:{}", MARKET_DATA_IP.to_string(), port))
        .expect("Failed to create server");
    println!(
        "[INFO]: UDP server on {}:{}",
        MARKET_DATA_IP.to_string(),
        port
    );
    loop {
        server.next_tick().unwrap();
        for server_event in server.drain_events() {
            println!("Server: Incoming event {:?}", server_event);
        }

        refresh_market_data(&mut provider, &recv_price);
        // ticker + bid + bid size + ask + ask size
        let curr_prices: &HashMap<String, PriceInfo> = provider.get_symb_to_prices();
        // let mut data : [u8] = [0; 36*curr_prices.len()];
        let mut data: Vec<u8> = vec![0; 36 * curr_prices.len()];
        // let mut data = [0; curr_prices.len()*(4 + 8 + 8 + 8 + 8)];
        for (ticker, price_info) in curr_prices.iter() {
            println!("sending {:?} for {}", price_info, ticker);
            // set top-level market data
            let ticker_bytes = ticker.as_bytes();
            let num: u32 = NetworkEndian::read_u32(ticker_bytes);
            data.write_u32::<NetworkEndian>(num).unwrap();
            data.write_u64::<NetworkEndian>(price_info.best_bid)
                .unwrap();
            data.write_u64::<NetworkEndian>(price_info.bid_size)
                .unwrap();
            data.write_u64::<NetworkEndian>(price_info.best_ask)
                .unwrap();
            data.write_u64::<NetworkEndian>(price_info.ask_size)
                .unwrap();

            // NetworkEndian::write_u32(&mut data, num);
            // NetworkEndian::write_u64(&mut data, price_info.best_bid);
            // NetworkEndian::write_u64(&mut data, price_info.bid_size);
            // NetworkEndian::write_u64(&mut data, price_info.best_ask);
            // NetworkEndian::write_u64(&mut data, price_info.ask_size);
        }
        // let data_wrapped : Arc<[u8]> = Arc::from(data.iter().cloned().map(|x| x as u8).collect::<Vec<u8>>().into_boxed_slice());
        println!("{:?}", data);
        let data_wrapped: Arc<[u8]> = Arc::from(data.into_boxed_slice());

        server.send_data(&data_wrapped, reliudp::MessageType::KeyMessage);
        ::std::thread::sleep(::std::time::Duration::from_millis(1));
    }
}

fn refresh_market_data(provider: &mut MarketDataProvider, recv_price: &Receiver<PriceInfo>) -> () {
    for i in 1..100 {
        let recv_next = recv_price.try_recv();
        if recv_next.is_err() {
            break;
        }
        let new_price_info = recv_next.unwrap();
        // let new_price_info = recv_price.recv().expect("no data in price info queue");
        println!("new price info: {:?}", new_price_info);
        provider.update_price(new_price_info);
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
*/
