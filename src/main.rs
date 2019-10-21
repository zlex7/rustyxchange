#[macro_use]
extern crate lazy_static;
extern crate getset;
extern crate byteorder;
extern crate reliudp;
extern crate serde;

use std::{str, u32, thread};
use std::collections::{HashSet, HashMap};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::{BufRead, BufReader};
use std::fs::File;

// market data implementation
mod market_data;
use market_data::start_market_data_server;
// all the types that will be shared across implementations
mod types;
use types::*;

// matching engine implementation
mod matching_engine;
use matching_engine::process_orders;

// exchange gateway implementation
mod gateway;
use gateway::Gateway;

// TODO: need some mechanism that will save/resend data in case of failure
// TODO: we will need some kind of database to store credentials!!!
// TODO: error handling so we don't die on bad input
// TODO: journaling??

const GATEWAY_IP: &'static str = "0.0.0.0";
const GATEWAY_PORT: u32 = 8888;
const ACCOUNTS_FILE : &'static str = "accounts.json";
const SYMBOLS_FILE : &'static str = "symbols.txt";

lazy_static! {
    pub static ref ACCOUNTS: HashMap<String, Account> = load_user_accounts(ACCOUNTS_FILE);
    pub static ref SYMBOLS: HashMap<String, Symbol> = load_symbols(SYMBOLS_FILE);
}

fn load_user_accounts(_filename : &str) -> HashMap<String, Account> {
    return HashMap::new();
}

fn load_symbols(_filename : &str) -> HashMap<String, Symbol> {
    let rdr = BufReader::new(File::open(SYMBOLS_FILE).expect("[ERROR] couldn't open symbols file"));
    let mut symbols: HashMap<String, Symbol> = HashMap::new();
    for line in rdr.lines() {
        let line = line.unwrap();
        println!("{}", line);
        symbols.insert(line.clone(), Symbol::new(line));
    } 

    symbols
}

fn main() {
    // create channels for orders
    let (order_sender, order_receiver): (Sender<Cmd>, Receiver<Cmd>) = channel();
    let (md_sender, md_receiver): (Sender<PriceInfo>, Receiver<PriceInfo>) = channel();

    // TODO: spawn thread for market data distribution
    let mut symbols = HashSet::new();
    for symbol in SYMBOLS.values() {
        symbols.insert(symbol.clone());
    }

    // spawn thread for matching engine, pass receiver channel into matching engine
    thread::spawn(|| {
        process_orders(md_sender, order_receiver);
    });

    thread::spawn(|| {
        start_market_data_server(md_receiver);
    });
    // let md: MarketData = MarketData::new(MARKET_DATA_IP, MARKET_DATA_PORT, md_receiver);
    // thread::Builder::new().name("md".to_string()).spawn(move || {
    //     md.run();
    // }).expect("[ERROR] failed to create market data thread");

    // TODO: spawn thread for market data server

    // initialize gateway, start TCP server
    let gateway: Gateway = Gateway::new(GATEWAY_IP, GATEWAY_PORT, order_sender);
    gateway.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: test init functions
    #[test]
    fn test_add() {

    }
}