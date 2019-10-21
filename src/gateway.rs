use byteorder::{ByteOrder, NetworkEndian};
use std::convert::TryInto;
use std::error::Error;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::{str, thread, u32, fmt, error};

use super::SYMBOLS;
use types::*;

pub struct Gateway {
    ip_addr: &'static str,
    port: u32,
    order_channel: Sender<Cmd>,
}

impl Gateway {
    pub fn new(ip_addr: &'static str, port: u32, order_channel: Sender<Cmd>) -> Self {
        Gateway {
            ip_addr: ip_addr,
            port: port,
            order_channel: order_channel,
        }
    }

    pub fn run(&self) {
        // Start gateway thread, open tcp connection
        let listener = TcpListener::bind(format!("{}:{}", self.ip_addr, self.port))
            .expect("[ERROR] couldn't connect to server");
        println!("[INFO] gateway started on {}:{}", self.ip_addr, self.port);

        // FIXME: how would we handle DDoS attack?
        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    // TODO: auethenticate client
                    let addr = s.peer_addr().unwrap();
                    println!("[INFO] new connection: {}", addr);

                    let client: Client = Client::new(s, self.order_channel.clone());
                    thread::Builder::new().name(format!("{}", addr)).spawn(move || {
                        client.run();
                    }).expect("[ERROR] failed to create client thread");
                }
                Err(e) => {
                    println!("[ERROR] client connection failed: {}", e);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct InvalidRWSize;

impl fmt::Display for InvalidRWSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid read or write size from buffer")
    }
}

impl error::Error for InvalidRWSize {
    fn description(&self) -> &str {
        "invalid read or write size from buffer"
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

struct Client {
    stream: TcpStream,
    order_channel: Sender<Cmd>,
    sender: Sender<OrderStatus>,
    receiver: Receiver<OrderStatus>,
}

impl Client {
    fn new(stream: TcpStream, order_channel: Sender<Cmd>) -> Self {
        let (sender, receiver): (Sender<OrderStatus>, Receiver<OrderStatus>) = channel();

        // set timeout to none -- we will handle dead connections ourselves
        stream
            .set_read_timeout(None)
            .expect("[ERROR] failed to set read timeout to None");

        Client {
            stream: stream,
            order_channel: order_channel,
            sender: sender,
            receiver: receiver,
        }
    }

    fn run(&self) {
        let mut reader = BufReader::new(
            self.stream
                .try_clone()
                .expect("[ERROR] failed to clone stream"),
        );

        let mut writer = BufWriter::new(
            self.stream
                .try_clone()
                .expect("[ERROR] failed to clone stream"),
        );

        loop {
            if !reader.fill_buf().unwrap_or(&[] as &[u8]).is_empty() {
                match self.recv_order(&mut reader) {
                    Ok(cmd) => {
                        self.order_channel.send(cmd).expect("[ERROR] order channel was dropped");
                    }
                    Err(e) => {
                        println!("[ERROR] failed to process order: {}", e);
                    }
                }
            }

            // FIXME: will this be inefficient?
            while let Ok(order_status) = self.receiver.try_recv() {
                match self.send_status(&mut writer, order_status) {
                    Err(e) => {
                        println!("[ERROR] failed to send status: {}", e);
                    },
                    _ => {}
                }
            }
        }

        /*
        self.stream
            .shutdown(Shutdown::Both)
            .expect("[ERROR] failed to shutdown tcp stream");
        */
    }

    fn recv_order(&self, reader: &mut BufReader<TcpStream>) -> Result<Cmd, Box<dyn Error>> {
        // read the first byte
        let mut data = [0 as u8; 1];
        reader.read_exact(&mut data)?;

        let size: usize = data[0] as usize;
        let mut data = vec![0 as u8; size];
        let read_size = reader.read(&mut data)?;
        if read_size == 0 || read_size != size {
            return Err(InvalidRWSize.into())
        }

        let order = self.data_to_struct(data.as_slice())?;
        Ok(order)
    }

    fn send_status(&self, writer: &mut BufWriter<TcpStream>, order_status: OrderStatus) -> Result<(), Box<dyn Error>> {
        let mut data: Vec<u8> = vec![];

        // TODO: check these byte values
        match order_status {
            OrderStatus::Filled(order_id, price) => {
                data.push(13 as u8);
                data.push(0 as u8);
                NetworkEndian::write_u32(&mut data, order_id);
                NetworkEndian::write_u64(&mut data, price);
            }
            OrderStatus::PartiallyFilled(order_id, quantity, price) => {
                data.push(21 as u8);
                data.push(1 as u8);
                NetworkEndian::write_u32(&mut data, order_id);
                NetworkEndian::write_u64(&mut data, quantity);
                NetworkEndian::write_u64(&mut data, price);
            }
            OrderStatus::Waiting(order_id) => {
                data.push(5 as u8);
                data.push(2 as u8);
                NetworkEndian::write_u32(&mut data, order_id);
            }
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
            }
            OrderStatus::Canceled(order_id) => {
                data.push(4 as u8);
                NetworkEndian::write_u32(&mut data, order_id);
            }
        };

        let size = data.len();
        let write_size = writer.write(data.as_slice())?;

        if write_size == 0 || write_size != size {
            return Err(InvalidRWSize.into())
        }

        Ok(())
    }

    fn data_to_struct(&self, data: &[u8]) -> Result<Cmd, Box<dyn Error>> {
        let cmd_type = CmdType::from_id(data[0] & 3);
        let account_id = u32::from_be_bytes(data[1..5].try_into()?);

        match cmd_type {
            CmdType::Execute => {
                let order_side = OrderSide::from_id(data[0] >> 2);
                let mut order_type = OrderType::from_id(data[5]);
                let ticker = str::from_utf8(&data[6..10])?;

                match order_type {
                    OrderType::Limit(ref mut thresh) => {
                        *thresh = NetworkEndian::read_u64(data[10..18].try_into()?);
                    }
                    OrderType::Stop(ref mut thresh) => {
                        *thresh = NetworkEndian::read_u64(data[10..18].try_into()?);
                    }
                    _ => {}
                };

                let quantity = u64::from_be_bytes(data[18..26].try_into()?);
                let symbol = SYMBOLS
                    .get(ticker)
                    .expect(&format!("[ERROR]: invalid ticker {} found", ticker)[..]);

                Ok(Cmd::Execute(OrderInfo::new(
                    account_id,
                    symbol,
                    order_type,
                    order_side,
                    quantity,
                    self.sender.clone(),
                )))
            }
            CmdType::Status => {
                let order_id = u32::from_be_bytes(
                    data[5..9]
                        .try_into()
                        .expect("[ERROR]: incorrect number of elements in slice"),
                );

                Ok(Cmd::Status(StatusInfo::new(
                    account_id,
                    order_id,
                    self.sender.clone(),
                )))
            }
            CmdType::Cancel => {
                let order_id = u32::from_be_bytes(
                    data[5..9]
                        .try_into()
                        .expect("[ERROR]: incorrect number of elements in slice"),
                );

                Ok(Cmd::Cancel(CancelInfo::new(
                    account_id,
                    order_id,
                    self.sender.clone(),
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {

    }
}