
static MARKET_DATA_SERVER_HOST : String = "0.0.0.0";
static MARKET_DATA_SERVER_PORT : String = "61244";

fn get_available_port() -> Option<u16> {
    (8000..62000)
        .find(|port| port_is_available(*port))
}

pub fn start_market_data_server(recv_price Receiver<PriceInfo>) -> () {
    let provider = MarketDataProvider::new();
    let port = get_available_port().expect("not a single port from 8000-62000 is open???");
    let mut server = reliudp::RUdpServer::new(format!("{}:{}",MARKET_DATA_SERVER_HOST,port).expect("Failed to create server");
    loop {
        server.next_tick()?;
        for server_event in server.drain_events() {
            println!("Server: Incoming event {:?}", server_event);
        }
        // ticker + bid + bid size + ask + ask size
        let curr_prices = provider.get_symb_to_prices();
        let mut raw = [0; curr_prices.len()*(4 + 8 + 8 + 8 + 8)];
        for (ticker, price_info) in curr_prices.iter() {
            raw[0:]
        }

        server.send_data(&big_message, reliudp::MessageType::KeyMessage);
        // ::std::thread::sleep(::std::time::Duration::from_millis(5));
    }
}

fn refresh_market_data(provider: MarketDataProvider, recv_price: &mut Receiver<PriceInfo>) -> () {
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

