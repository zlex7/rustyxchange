use std::collections::HashMap;
use std::collections::BTreeMap;

// something you can buy or sell
// FIXME: is this necessary? if we read in from a file we can just have a vec of strings representing symbols
struct Symbol {
    ticker: String
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderSide {
    Buy,
    Sell
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderType {
    Market,
    Limit,
    Stop,
    Cancel
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderStatus {
    Filled,
    PartiallyFilled,
    Rejected,
    Canceled
}

struct Order {
    symbol: Symbol,
    type: OrderType,
    side: OrderSide,
    price: u64,
    quantity: u64,
    account_id: u32
}

impl Order {
    fn new(symbol: Symbol, type: OrderType, side: OrderSide, price: u64, quantity: u64, account_id: u32) -> Order {
        Order {
            symbol: symbol,
            type: type,
            side: side,
            price: price,
            quantity: quantity,
            account_id: account_id
        }
    }
}


struct OrderBook {
    bids: BTreeMap<u32, Order>,
    asks: BTreeMap<u32, Order>
}

impl OrderBook {
    fn new() -> OrderBook {
        OrderBook { 
            bids: BTreeMap<u32, Order>::new(),
            asks: BTreeMap::new()
        }
    }

    fn get_best_bid(&self) -> f64 {
        self.bids.iter().next()?
    }

    fn get_best_ask(&self) -> f64 {
        self.asks.iter().next()?
    }

    fn new_bid(&self) {
        // TODO: more parameters for new bid order
    }

    fn new_ask(&self) {
        // TODO: more parameters for new ask order
    }
}

struct MatchingEngine {
    // map from symbol to orderbook
    order_books: HashMap<String, OrderBook>,
    order_queue: Vec<Order>
}

impl MatchingEngine {
    // FIXME: should matching engine be a static class, or should it have its own instances?
    pub fn new() -> MatchingEngine {
        MatchingEngine {

        }
    }

    // TODO: how should this method be structured? Should all order types be handled in one?
    pub fn place_order(&self, order: Order) {
        // market orders are executed immediately if possible, otherwise added to queue
        // limit orders are added to queue and executed when the price is reached and its turn comes in queue
        // TODO: how to implement stop orders?
    }
}
