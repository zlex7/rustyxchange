#[macro_use]
extern crate lazy_static;
extern crate csv;
extern crate byteorder;
#[macro_use]
extern crate getset;

use std::{str, u32, thread, fs};
use std::io::{BufReader, Read};
use std::collections::HashMap;
use std::convert::TryInto;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::sync::mpsc::{Sender, Receiver, channel};
use byteorder::{NetworkEndian, ByteOrder};
// FIXME: what is the Endianess?
// use std::fs::File;

// all the types that will be shared across implementations
mod types;
use types::*;

// matching engine implementation
mod matching_engine;
use matching_engine::process_orders;

// TODO: need some mechanism that will save/resend data in case of failure
// TODO: we will need some kind of database to store credentials!!!
// TODO: error handling so we don't die on bad input
// TODO: journaling??

const IP_ADDR: &'static str = "0.0.0.0";
const PORT: u32 = 8888;
const ACCOUNTS_FILE : &'static str = "accounts.db";
const SYMBOLS_FILE : &'static str = "symbols.txt";

lazy_static! {
    pub static ref ACCOUNTS: HashMap<String, Account> = load_user_accounts(ACCOUNTS_FILE);
    pub static ref SYMBOLS: HashMap<String, Symbol> = load_symbols(SYMBOLS_FILE);
}

fn load_user_accounts(filename : &str) -> HashMap<String, Account> {
    // TODO: handle errors better here
    // TODO: the accounts file is potentially very large, redo this to be more efficient
    let mut reader = csv::Reader::from_reader(fs::read_to_string(filename).unwrap().as_bytes());
    let mut accounts = HashMap::new();
    /*
    for account in reader.deserialize() {
        // TODO: create account
    }
    */

    return accounts;
}

/// reads all the symbols from a file and returns a list
/// 
/// # Parameters
/// 
/// * `filename` - the name of the file to read from
fn load_symbols(filename : &str) -> HashMap<String, Symbol> {
    // TODO: handle errors better here
    let mut reader = csv::Reader::from_reader(fs::read_to_string(filename).unwrap().as_bytes());
    let mut symbols = HashMap::new();
    /*
    for symbol in reader.deserialize() {
        // TODO: create symbol
    }
    */

    return symbols;
}

fn main() {
    // INIT state:
    //      read symbols file
    //      load state from journal (if no journal, run init_new function). state is order book, last market data sent.
    //      Account data for users -> usernames, passwords, position data. -> done by lazy_static
    let symbols = load_symbols(SYMBOLS_FILE);
    // load previous state of matching engine...

    // create channels for orders and subscriptions
    let (order_sender, order_receiver): (Sender<OrderInfo>, Receiver<OrderInfo>) = channel();
    let (sub_sender, sub_receiver): (Sender<SubscribeInfo>, Receiver<SubscribeInfo>) = channel();

    // TODO: spawn thread for market data distribution

    // spawn thread for matching engine, pass receiver channel into matching engine
    thread::spawn(move|| {
        process_orders(order_receiver)
    });

    // Start gateway thread, open tcp connection
    let listener = TcpListener::bind(format!("{}:{}", IP_ADDR, PORT)).expect("[ERROR]: Couldn't connect to the server...");
    println!("[INFO]: Server listening on port {}", PORT);

    // TODO: each request needs an authentication header
    // TODO: we only need to authenticate once for each connection
    // will we need to wrap this in a loop {}? not really sure how .incoming() works
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // TODO: how should we handle people spamming connections?
                println!("New connection: {}", stream.peer_addr().unwrap());
                // thread::spawn(move|| {
                //     handle_client(stream, order_sender.clone(), sub_sender.clone())
                // });
            }
            Err(e) => {
                println!("[ERROR]: {}", e);
            }
        }
    }

    drop(listener);
}

/// handle data incoming from a client connection
/// TODO: handle subscribe actions
fn handle_client(mut stream: TcpStream, order_sender: Sender<OrderInfo>, sub_sender: Sender<SubscribeInfo>) {
    // set timeout to none -- we will handle dead connections ourselves
    stream.set_read_timeout(None);
    let mut reader = BufReader::new(stream);

    loop {
        let mut data = [0 as u8; 1];

        // read the first byte
        reader.read_exact(&mut data);
        let size: usize = data[0] as usize;

        let mut data = vec![0 as u8; size];
        match reader.read(&mut data) {
            Ok(read_size) => {
                if read_size == 0 {
                    break;
                }

                if read_size != size {
                    // TODO: error
                    panic!("[ERROR]: expected to read {} bytes, read {} instead", size, read_size);
                }

                match data_to_struct(data.as_slice()) {
                    NetworkData::Order(order_info) => {
                        // TODO: additional behavior we need when sending order info
                        if order_sender.send(order_info).is_err() {
                            panic!("[ERROR]: channel to matching engine was dropped");
                        }
                    },
                    NetworkData::Subscribe(sub_info) => {
                        if sub_sender.send(sub_info).is_err() {
                            panic!("[ERROR]: channel to market data was dropped");
                        }
                    },
                    NetworkData::Status(status_info) => {
                        // TODO: expose get status function in market engine
                    },
                    NetworkData::Cancel(cancel_info) => {

                    }
                }                                 
            },
            Err(e) => {
                println!("[ERROR]: {}", e);
            }
        }
    }

    // stream.shutdown(Shutdown::Both); 
}

fn data_to_struct(data: &[u8]) -> NetworkData {
    let cmd_type = CmdType::from_id(data[0] & 3);
    // FIXME: assuming format is big endian, may be incorrect
    let account_id = u32::from_be_bytes(data[1..5].try_into().expect("[ERROR]: incorrect number of elements in slice"));
    match cmd_type {
        CmdType::Execute => {
            // TODO: verify values read are correct
            let order_side = OrderSide::from_id(data[0] >> 2);
            let mut order_type = OrderType::from_id(data[5]);
            let ticker = str::from_utf8(&data[6..10]).expect("[ERROR]: failed to convert byte array to str");

            match order_type {
                OrderType::Limit(ref mut thresh) => {
                    *thresh = NetworkEndian::read_u64(data[10..18].try_into().expect("[ERROR]: incorrect number of elements in slice"));
                },
                OrderType::Stop(ref mut thresh) => {
                    *thresh = NetworkEndian::read_u64(data[10..18].try_into().expect("[ERROR]: incorrect number of elements in slice"));
                }
                OrderType::Market => {}
            };

            let quantity = u32::from_be_bytes(data[18..22].try_into().expect("[ERROR]: incorrect number of elements in slice"));
            let symbol = SYMBOLS.get(ticker).expect(&format!("[ERROR]: invalid ticker {} found", ticker)[..]);

            NetworkData::Order(OrderInfo {
                account_id: account_id,
                symbol: symbol,
                order_type: order_type,
                side: order_side,
                quantity: quantity,
            })
        },
        CmdType::Subscribe => {
            let ticker = str::from_utf8(&data[6..10]).expect("[ERROR]: failed to convert byte array to str");
            let symbol = SYMBOLS.get(ticker).expect(&format!("[ERROR]: invalid ticker {} found", ticker)[..]);

            NetworkData::Subscribe(SubscribeInfo {
                account_id: account_id,
                symbol: symbol
            })
        },
        CmdType::Status => {
            let order_id = u32::from_be_bytes(data[5..9].try_into().expect("[ERROR]: incorrect number of elements in slice"));
            
            NetworkData::Status(StatusInfo {
                account_id: account_id,
                order_id: order_id
            })
        },
        CmdType::Cancel => {
            let order_id = u32::from_be_bytes(data[5..9].try_into().expect("[ERROR]: incorrect number of elements in slice"));

            NetworkData::Cancel(CancelInfo {
                account_id: account_id,
                order_id: order_id
            })
        },
    }
}

enum NetworkData {
    Order(OrderInfo),
    Subscribe(SubscribeInfo),
    Status(StatusInfo),
    Cancel(CancelInfo) 
}


// TODO: unit tests to make sure functions are working correctly
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {

    }
}