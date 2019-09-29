
#[macro_use]
extern crate lazy_static;
extern crate getset;
extern crate csv;
extern crate byteorder;

use std::{str, u32, thread, fs};
use std::io::{BufReader, BufWriter, Read, Write};
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
    let reader = csv::Reader::from_reader(fs::read_to_string(filename).unwrap().as_bytes());
    let accounts = HashMap::new();

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
    let reader = csv::Reader::from_reader(fs::read_to_string(filename).unwrap().as_bytes());
    let symbols = HashMap::new();

    /*
    for symbol in reader.deserialize() {
        // TODO: create symbol
    }
    */

    return symbols;
}

fn main() {
    // TODO: load previous state of matching engine...

    // create channels for orders and subscriptions
    let (order_sender, order_receiver): (Sender<OrderInfo>, Receiver<OrderInfo>) = channel();
    let (sub_sender, sub_receiver): (Sender<SubscribeInfo>, Receiver<SubscribeInfo>) = channel();

    // TODO: spawn thread for market data distribution

    // spawn thread for matching engine, pass receiver channel into matching engine
    thread::spawn(|| {
        process_orders(order_receiver)
    });

    // Start gateway thread, open tcp connection
    let listener = TcpListener::bind(format!("{}:{}", IP_ADDR, PORT)).expect("[ERROR]: Couldn't connect to the server...");
    println!("[INFO]: Server listening on port {}", PORT);

    // FIXME: how should we handle people spamming connections?
    // will we need to wrap this in a loop {}? not really sure how .incoming() works
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // TODO: authenticate the client here, since we only need to authenticate once
                println!("New connection: {}", stream.peer_addr().unwrap());
                let o_sender = order_sender.clone();
                let s_sender = sub_sender.clone();

                thread::spawn(move || {
                    handle_client(stream, o_sender, s_sender);
                });
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
fn handle_client(stream: TcpStream, order_sender: Sender<OrderInfo>, sub_sender: Sender<SubscribeInfo>) {
    // set timeout to none -- we will handle dead connections ourselves
    stream.set_read_timeout(None);
    let mut reader = BufReader::new(stream.try_clone().expect("[ERROR]: failed to clone stream"));

    let (response_sender, response_receiver): (Sender<OrderStatus>, Receiver<OrderStatus>) = channel();
    let stream_copy = stream.try_clone().expect("[ERROR]: failed to clone stream");
    // spawn response thread
    thread::spawn(move|| {
        handle_response(stream_copy, response_receiver);
    });

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

                match data_to_struct(data.as_slice(), response_sender.clone()) {
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

    stream.shutdown(Shutdown::Both); 
}

fn handle_response(stream: TcpStream, response_receiver: Receiver<OrderStatus>) {
    let mut writer = BufWriter::new(stream);
    loop {
        let order_status = response_receiver.recv().expect("[ERROR]: channel from matching engine was dropped");
        let mut data: Vec<u8> = vec![];
        match order_status {
            OrderStatus::Filled(order_id, price) => {
                data.push(13 as u8);
                data.push(0 as u8);
                NetworkEndian::write_u32(&mut data, order_id);
                NetworkEndian::write_u64(&mut data, price);
            },
            OrderStatus::PartiallyFilled(order_id, quantity, price) => {
                data.push(17 as u8);
                data.push(1 as u8);
                NetworkEndian::write_u32(&mut data, order_id);
                NetworkEndian::write_u32(&mut data, quantity);
                NetworkEndian::write_u64(&mut data, price);
            },
            OrderStatus::Waiting(order_id) => {
                data.push(5 as u8);
                data.push(2 as u8);
                NetworkEndian::write_u32(&mut data, order_id);
            },
            OrderStatus::Rejected(order_id, reason) => {
                data.push(13 as u8);
                data.push(3 as u8);
                NetworkEndian::write_u32(&mut data, order_id);
                // len returns a usize, which can be either a u32 or u64. for simplicity, assume it is a u64
                NetworkEndian::write_u64(&mut data, reason.len() as u64);
                // append the message after the payload
                for byte in reason.as_bytes() {
                    data.push(*byte);
                }
            },
            OrderStatus::Canceled(order_id) => {
                data.push(4 as u8);
                NetworkEndian::write_u32(&mut data, order_id);
            }
        };

        let size = data.len();
        match writer.write(data.as_slice()) {
            Ok(write_size) => {
                if write_size != size {
                    println!("[ERROR]: expected to write {} bytes, wrote {} instead", size, write_size);
                }
            },
            Err(e) => {
                println!("[ERROR]: failed to write response to tcp stream");
            }
        };
    }
}

fn data_to_struct(data: &[u8], response_sender: Sender<OrderStatus>) -> NetworkData {
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
                _ => {}
            };

            let quantity = u32::from_be_bytes(data[18..22].try_into().expect("[ERROR]: incorrect number of elements in slice"));
            let symbol = SYMBOLS.get(ticker).expect(&format!("[ERROR]: invalid ticker {} found", ticker)[..]);

            NetworkData::Order(OrderInfo::new(account_id,symbol,order_type,order_side,quantity,response_sender))
        },
        CmdType::Subscribe => {
            let ticker = str::from_utf8(&data[6..10]).expect("[ERROR]: failed to convert byte array to str");
            let symbol = SYMBOLS.get(ticker).expect(&format!("[ERROR]: invalid ticker {} found", ticker)[..]);

            NetworkData::Subscribe(SubscribeInfo::new(account_id, symbol))
        },
        CmdType::Status => {
            let order_id = u32::from_be_bytes(data[5..9].try_into().expect("[ERROR]: incorrect number of elements in slice"));
            
            NetworkData::Status(StatusInfo::new(account_id, order_id, response_sender))
        },
        CmdType::Cancel => {
            let order_id = u32::from_be_bytes(data[5..9].try_into().expect("[ERROR]: incorrect number of elements in slice"));

            NetworkData::Cancel(CancelInfo::new(account_id, order_id, response_sender))
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