use std::collections::HashMap;
use std::collections::BTreeMap;

struct Account {
    // positions is map<symbol, int>
    positions : HashMap,
    // amount of deposited money
    balance : f64,
    // unique identifier
    id : u32
}

// something you can buy or sell
struct Symbol {
    ticker : String
}

struct Order {
    symbol : Symbol,
    type : OrderType,
    side : OrderSide,
    price : u64,
    quantity : u64,
    account_id : u32
}

enum OrderSide {

}

enum OrderType {

}

enum OrderStatus {

}

struct OrderBook {
    bids : BTreeMap,
    asks : BTreeMap
    // can also just get from treemap
    best_bid : f64,
    best_bid : f64
}

struct MatchingEngine {
    // map from symbol to orderbook
    order_books : HashMap,
    market_order_queue : Vec,
    limit_order_queue : Vec
}
