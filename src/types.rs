use std::collections::HashMap;
#[macro_use]
use getset::{Getters};

pub trait FromId {
    fn from_id(id: u8) -> Self;
}

// ENUMS //

/// the type of command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdType {
    Execute,
    Subscribe,
    Status,
    Cancel,
    // TODO: positions sizes?
}

impl FromId for CmdType {
    fn from_id(id: u8) -> CmdType {
        match id {
            0 => CmdType::Execute,
            1 => CmdType::Subscribe,
            2 => CmdType::Status,
            3 => CmdType::Cancel,
        }
    }
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
            1 => OrderSide::Sell
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
            2 => OrderType::Stop(0 as u64)
        }
    }
}

/// 4 main types of statuses
/// * Filled - all of order was matched in exchange (# of shares/quantity)
/// * Partially Filled - part of order was matched in exchange
/// * Rejected - order was rejected for some reason, which will be specified
/// * Canceled - order was canceled
#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Filled(u64),
    PartiallyFilled(u32, u64),
    Waiting,
    Rejected(String),
    Canceled
}

// STRUCTS //

#[derive(Getters, Clone)]
pub struct Symbol {
    #[get = "pub"]
    ticker: String,
    // TODO: other metadata
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

pub struct SubscribeInfo {
    account_id: u32,
    symbol: &'static Symbol
}

pub struct StatusInfo {
    account_id: u32,
    order_id: u32
}

pub struct CancelInfo {
    account_id: u32,
    order_id: u32
}

pub struct OrderInfo {
    account_id: u32,
    symbol: &'static Symbol,
    order_type: OrderType,
    side: OrderSide,
    quantity: u32,
}

/// A struct containing all the information about a single order
// #[derive(Getters)]
// #[get = "pub"] // By default add a pub getting for all fields.
#[derive(Clone)]
pub struct Order {
    pub id: u32,
    pub account_id: u32,
    // #[get = "pub"]
    pub symbol: Symbol,
    // #[get = "pub"]
    pub order_type: OrderType,
    // #[get = "pub"]
    pub side: OrderSide,
    pub quantity: u32,
    pub remaining_quantity: u32,
    pub cost: u64
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

    pub fn get_status_based_on_fill(&self) -> OrderStatus {
        if self.remaining_quantity == self.quantity {
            return OrderStatus::Waiting;
        } else if self.remaining_quantity == 0 {
            return OrderStatus::Filled(self.cost);
        } else {
            return OrderStatus::PartiallyFilled(self.quantity - self.remaining_quantity, self.cost);
        }
    }

    pub fn fill_shares(&mut self, num_filled : u32, cost_per_share : u64) -> () {
        if num_filled > self.remaining_quantity {
            panic!("can't fill shares > curr quantity");
        }
        self.remaining_quantity -= num_filled;
        self.cost += (num_filled as u64) * cost_per_share;
    }

    pub fn is_fully_filled(self) -> bool {
        return self.remaining_quantity == 0;
    }
}


// UNUSED CODE //
/*
*/

