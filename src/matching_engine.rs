use std::collections::HashMap;
use std::collections::BTreeMap;
use std::sync::mpsc::Receiver;

use types::*;

/// a struct containing a list of open bids and asks
struct OrderBook {
    bids: BTreeMap<u32, Order>,
    asks: BTreeMap<u32, Order>
}

impl OrderBook {
    fn new() -> OrderBook {
        OrderBook { 
            bids: BTreeMap::new(),
            asks: BTreeMap::new()
        }
    }

    fn get_best_bid(&self) -> f64 {
        // TODO: get the best bid in the bids map
        0 as f64
    }

    fn get_best_ask(&self) -> f64 {
        // TODO: get the best ask in the asks map
        0 as f64
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
    // order_queue: Vec<Order>
    order_books: HashMap<String, OrderBook>,
}

impl MatchingEngine {
    // FIXME: should matching engine be a static class, or should it have its own instances?
    fn new() -> MatchingEngine {
        MatchingEngine {
            order_books: HashMap::new(),
        } 
    }

    // TODO: how should this method be structured? Should all order types be handled in one?
    fn process_order(&self, order: Order) {
        // market orders are executed immediately if possible, otherwise added to queue
        // limit orders are added to queue and executed when the price is reached and its turn comes in queue
        // TODO: how to implement stop orders?
    }

    pub fn limit_order(&self, order: Order) {
        
    }

    /*
    pub fn market_order(&self, order: Order) -> OrderStatus {
        match order.side {
            BUY => {
                return market_order_generic(order, asks, market_bids);
            }
            SELL => {
                return market_order_generic(order, bids, market_asks);
            }
        }
    }

    fn market_order_generic(order: Order, opposite_limit_orders: BTreeMap, market_orders: Vec<Order>) -> OrderStatus {
        if opposite_limit_orders.len() == 0 {
            market_orders.push(order);
            return OrderStatus::Waiting;
        } else {
            let total_filled : u32 = 0;
            let total_cost : f64 = 0;
            'outer: for (price, opposite_order_lst) in opposite_limit_orders.iter() {
                for opposite_order in &mut opposite_order_lst {
                    let exec_price = opposite_order.price;
                    let q_filled = min(order.quantity - q_filled, opposite_order.quantity);
                    opposite_order.fill_shares(q_filled, exec_price);
                    order.fill_shares(q_filled, exec_price);
                    // if ask was filled
                    if opposite_order.is_fully_filled() {
                        opposite_order_lst.pop_front();
                    }
                    // if current order has been filled
                    if order.is_fully_filled() {
                        break 'outer;
                    }
                }
            }
            if ! order.is_fully_filled() {
                market_orders.push(order);
            }
            return order.get_status_based_on_fill();
        }
    }
    */
}

pub fn process_orders(recv: Receiver<OrderInfo>) {
    // let order_book = self.order_books.get(order.symbol);
    let matching_engine: MatchingEngine = MatchingEngine::new();
    loop {
        // TODO:
    }
}