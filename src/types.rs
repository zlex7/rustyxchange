use getset::{Getters};
use std::sync::mpsc::{Sender};
use std::collections::HashMap;

////////////
// TRAITS //
////////////

pub trait FromId {
    fn from_id(id: u8) -> Self;
}

///////////
// ENUMS //
///////////

/// the type of command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdType {
    Execute,
    Status,
    Cancel,
    // TODO: positions sizes?
}

impl FromId for CmdType {
    // Result<CmdType, &'static' str>
    fn from_id(id: u8) -> CmdType {
        match id {
            0 => CmdType::Execute,
            2 => CmdType::Status,
            3 => CmdType::Cancel,
            _ => panic!("command id does not exist")
        }
    }
}

pub enum Cmd {
    Execute(OrderInfo),
    Status(StatusInfo),
    Cancel(CancelInfo)
}

/// an order can either be a buy order or sell order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell
}

impl FromId for OrderSide {
    fn from_id(id: u8) -> OrderSide {
        match id {
            0 => OrderSide::Buy,
            1 => OrderSide::Sell,
            _ => panic!("order side does not exist")
        }
    }
}

/// 4 main kinds of orders
/// * Market Order - buy at market price
/// * Limit Order - only buy if price meets threshold, specify the limit price
/// * Stop Order - converts to market when threshold reached, specify the stop price
/// * Cancel Order - cancels a sent order
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    Market,
    Limit(u64),
    Stop(u64)
}

impl FromId for OrderType {
    fn from_id(id: u8) -> OrderType {
        match id {
            0 => OrderType::Market,
            1 => OrderType::Limit(0 as u64),
            2 => OrderType::Stop(0 as u64),
            _ => panic!("order type does not exist")
        }
    }
}

/// 4 main types of statuses
/// * Filled - all of order was matched in exchange (# of shares/quantity): order_id, price
/// * Partially Filled - part of order was matched in exchange: order_id, quantity, price
/// * Waiting - order has not been filled and is in order book: order_id
/// * Rejected - order was rejected for some reason, which will be specified: order_id, message
/// * Canceled - order was canceled: order_id
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum OrderStatus {
    Filled(u32, u64),
    PartiallyFilled(u32, u64, u64),
    Waiting(u32),
    Rejected(u32, &'static str),
    Canceled(u32)
}

/////////////
// STRUCTS //
/////////////

#[derive(Getters, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Symbol {
    #[get = "pub"]
    ticker: String,
    // TODO: other metadata
}

impl Symbol {
    pub fn new(ticker: String) -> Self {
        Symbol {
            ticker: ticker
        }
    }
}

/// a struct containing important information about an account
// TODO: add getter/setter methods
pub struct Account {
    positions: HashMap<String, i64>,
    username: String,
    password: String,
    initial: u64,
    balance: u64,
    id: u32
}

impl Account {
    /// creates a new Account with the specified id and initial balance
    /// 
    /// # Arguments
    /// 
    /// * `id` - the unique account id for this account
    /// * `initial` - the initial balance that this account started with
    pub fn new(id: u32, initial: u64) -> Account {
        Account {
            positions: HashMap::new(),
            username: String::from(""),
            password: String::from(""),
            initial: initial,
            balance: 0,
            id: id
        }
    }

    /// returns the profit/loss of the account
    fn get_pl(self) -> u64 {
        return self.balance - self.initial;
    }
}

#[derive(Clone, Debug)]
pub struct PriceInfo {
    symbol: &'static Symbol,
    pub best_bid: u64,
    pub bid_size: u64,
    pub best_ask: u64,
    pub ask_size: u64
}

impl PriceInfo {
    pub fn new(symbol: &'static Symbol, best_bid: u64, bid_size: u64, best_ask: u64, ask_size: u64) -> PriceInfo {
        PriceInfo {
            symbol: symbol,
            best_bid: best_bid,
            bid_size: bid_size,
            best_ask: best_ask,
            ask_size: ask_size
        }
    }

    pub fn get_symbol(&self) -> &Symbol {
        &self.symbol
    }
}

/*
pub struct SubscribeInfo {
    account_id: u32,
    // pub ip: String,
    symbol: &'static Symbol
}

impl SubscribeInfo {
    pub fn new(account_id: u32, symbol: &'static Symbol) -> SubscribeInfo {
        SubscribeInfo {
            account_id: account_id,
            symbol: symbol
        }
    }
}
*/

pub struct StatusInfo {
    account_id: u32,
    order_id: u32,
    response_sender: Sender<OrderStatus>
}

impl StatusInfo {
    pub fn new(account_id: u32, order_id: u32, response_sender: Sender<OrderStatus>) -> StatusInfo {
        StatusInfo {
            account_id: account_id,
            order_id: order_id,
            response_sender: response_sender
        }
    }

    pub fn consume(self) -> (u32, u32, Sender<OrderStatus>) {
        (self.account_id, self.order_id, self.response_sender)
    }
}

pub struct CancelInfo {
    account_id: u32,
    order_id: u32,
    response_sender: Sender<OrderStatus>
}

impl CancelInfo {
    pub fn new(account_id: u32, order_id: u32, response_sender: Sender<OrderStatus>) -> CancelInfo {
        CancelInfo {
            account_id: account_id,
            order_id: order_id,
            response_sender: response_sender
        }
    }

    pub fn consume(self) -> (u32, u32, Sender<OrderStatus>) {
        (self.account_id, self.order_id, self.response_sender)
    }
}

pub struct OrderInfo {
    account_id: u32,
    symbol: &'static Symbol,
    order_type: OrderType,
    side: OrderSide,
    quantity: u64,
    response_sender: Sender<OrderStatus>
}

impl OrderInfo {
    pub fn new(account_id: u32,symbol: &'static Symbol,order_type: OrderType,order_side: OrderSide,quantity: u64,response_sender: Sender<OrderStatus>) -> OrderInfo {
        OrderInfo {
            account_id: account_id,
            symbol: symbol,
            order_type: order_type,
            side: order_side,
            quantity: quantity,
            response_sender: response_sender
        }
    }

    pub fn consume(self, order_id: u32) -> (Order, Sender<OrderStatus>) {
        (Order {
            id: order_id,
            account_id: self.account_id,
            symbol: self.symbol,
            order_type: self.order_type,
            side: self.side,
            quantity: self.quantity,
            remaining_quantity: self.quantity,
            cost: 0 as u64,
            is_canceled: false
        },
        self.response_sender)
    }
}



/// A struct containing all the information about a single order
// #[derive(Getters)]
// #[get = "pub"] // By default add a pub getting for all fields.
#[derive(Clone, Debug)]
pub struct Order {
    pub id: u32,
    pub account_id: u32,
    pub symbol: &'static Symbol,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: u64,
    pub remaining_quantity: u64,
    pub cost: u64,
    pub is_canceled: bool
}

impl Order {
    /// creates a new order from the given information
    /// 
    /// # Parameters
    /// 
    /// * symbol - the symbol of the security being traded
    /// * type - the type of order, as specified above
    /// * side - the side of the order, as specified above
    /// * quantity - the number of shares to transact
    /*
    pub fn new(order_id: u32, account_id: u32, symbol: Symbol, order_type: OrderType, side: OrderSide, quantity: u32) -> Order {
        Order {
            id: order_id,
            account_id: account_id,
            symbol: symbol,
            order_type: order_type,
            side: side,
            remaining_quantity: 0,
            quantity: quantity,
            cost: 0
        }
    }
    */

    // FIXME: status could also be rejected or canceled
    pub fn get_status_based_on_fill(&self) -> OrderStatus {
        if self.is_canceled {
            return OrderStatus::Canceled(self.id);
        } else if self.remaining_quantity == self.quantity {
            return OrderStatus::Waiting(self.id);
        } else if self.remaining_quantity == 0 {
            return OrderStatus::Filled(self.id, self.cost);
        } else {
            return OrderStatus::PartiallyFilled(self.id, self.quantity - self.remaining_quantity, self.cost);
        }
    }

    pub fn fill_shares(&mut self, num_filled : u64, cost_per_share : u64) -> () {
        if num_filled > self.remaining_quantity {
            panic!("can't fill shares > curr quantity");
        }
        self.remaining_quantity -= num_filled;
        self.cost += (num_filled as u64) * cost_per_share;
    }

    pub fn is_fully_filled(&self) -> bool {
        return self.remaining_quantity == 0;
    }
}


// UNUSED CODE //
/*
*/

