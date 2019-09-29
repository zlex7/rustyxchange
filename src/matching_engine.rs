use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::LinkedList;
use std::sync::mpsc::Receiver;
use std::cmp;

use types::*;

/// a struct containing a list of open bids and asks
struct OrderBook {
    bids: BTreeMap<u64, LinkedList<u32>>,
    asks: BTreeMap<u64, LinkedList<u32>>,
    market_bids: LinkedList<u32>,
    market_asks: LinkedList<u32>,
    orders: HashMap<u32, Order>
}

impl OrderBook {
    fn new() -> OrderBook {
        OrderBook { 
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            market_bids: LinkedList::new(),
            market_asks: LinkedList::new(),
            orders: HashMap::new()
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


    // order priority:
    //  1. market > limit
    //  2. timestamp
    

    //TODO: one problem we need to deal with is making appropiate variables mutable in Order struct
    pub fn order(&mut self, old_order : &Order) -> OrderStatus {
        // self.orders.insert(old_order.id, old_order.clone());
        // let order : &mut Order = self.orders.get_mut(&old_order.id).unwrap();
        let mut order = old_order.clone();
        let order_status = match order.order_type {
            OrderType::Market => self.market_order(&mut order),
            OrderType::Limit(price) => self.limit_order(&mut order, price),
            OrderType::Stop(price) => self.stop_order(&mut order, price)
        };
        self.orders.insert(order.id, order.clone());
        return order_status;
    }


    pub fn stop_order(&mut self, order : &mut Order, price: u64) -> OrderStatus {
        return OrderStatus::Waiting(order.id);
    }

    // fucks with internal data structures
    pub fn limit_order(&mut self, order : &mut Order, price: u64) -> OrderStatus {
        match order.side {
            BUY => {
                return OrderBook::limit_order_generic(order, price, &mut self.bids, &mut self.asks, &mut self.market_bids, &mut self.orders);
            }
            SELL => {
                return OrderBook::limit_order_generic(order, price, &mut self.asks, &mut self.bids, &mut self.market_asks, &mut self.orders);
            }
        }
    }

    // fucks with internal data structures
    pub fn market_order(&mut self, order: &mut Order) -> OrderStatus {
        match order.side {
            BUY => {
                return OrderBook::market_order_generic(order, &mut self.asks, &mut self.market_bids, &mut self.orders);
            }
            SELL => {
                return OrderBook::market_order_generic(order, &mut self.bids, &mut self.market_asks, &mut self.orders);
            }
        }
    }

    fn fill_on_opposite_limit_orders_lst(order: &mut Order,  price: u64, opposite_order_lst: &mut LinkedList<u32>, orders: &mut HashMap<u32,Order>) -> bool {
        let mut num_orders_to_remove = 0;
        for id in opposite_order_lst.iter_mut() {
            let opposite_order : &mut Order = orders.get_mut(id).unwrap();
            let q_filled = cmp::min(order.remaining_quantity, opposite_order.quantity);
            opposite_order.fill_shares(q_filled, price);
            order.fill_shares(q_filled, price);
            // if ask was filled
            if opposite_order.is_fully_filled() {
                num_orders_to_remove += 1;
            }
            // if current order has been filled
            if order.is_fully_filled() {
                break;
            }
        }
        for i in 0..num_orders_to_remove {
            opposite_order_lst.pop_front();
        }
        return order.is_fully_filled();
    }

    fn list_limit_order(order: &mut Order, price_per_share: u64, limit_orders: &mut BTreeMap<u64, LinkedList<u32>>) -> () {
        limit_orders.entry(price_per_share).or_insert(LinkedList::new());
        limit_orders.get_mut(&price_per_share).unwrap().push_back(order.id);
    }


    fn limit_order_generic(order: &mut Order, price_per_share: u64, same_side_limit_orders: &mut BTreeMap<u64, LinkedList<u32>>, opposite_limit_orders: &mut BTreeMap<u64, LinkedList<u32>>, market_orders: &mut LinkedList<u32>, orders: &mut HashMap<u32,Order>) -> OrderStatus {
        // prioritizing market orders
        let mut num_to_remove = 0;
        for id in market_orders.iter() {
            let m_order = orders.get_mut(id).unwrap();
            let q_filled = cmp::min(order.remaining_quantity, m_order.quantity);
            m_order.fill_shares(q_filled, price_per_share);
            order.fill_shares(q_filled, price_per_share);

            if m_order.is_fully_filled() {
                num_to_remove += 1;
            }
            if order.is_fully_filled() {
                break;
            }
        }
        
        for i in 0..num_to_remove {
            market_orders.pop_front();
        }
        if order.is_fully_filled() {
            return order.get_status_based_on_fill();
        }

        for (opposite_price, opposite_order_lst) in opposite_limit_orders.iter_mut() {
            if (order.side == OrderSide::Buy && *opposite_price > price_per_share) || (order.side == OrderSide::Sell && *opposite_price < price_per_share) {
                break;
            }
            let is_fully_filled = OrderBook::fill_on_opposite_limit_orders_lst(order, price_per_share, opposite_order_lst, orders);
            if is_fully_filled {
                    break;
            }
        }
        
        if ! order.is_fully_filled() {
            OrderBook::list_limit_order(order, price_per_share, same_side_limit_orders);
        }
        return order.get_status_based_on_fill();
    }

    //TODO: remove key from BTree when no orders left at that price.
    fn market_order_generic(order: &mut Order, opposite_limit_orders: &mut BTreeMap<u64, LinkedList<u32>>, market_orders: &mut LinkedList<u32>, orders: &mut HashMap<u32,Order>) -> OrderStatus {
        if opposite_limit_orders.len() == 0 {
            market_orders.push_back(order.id);
            return OrderStatus::Waiting(order.id);
        } else {
            for (price, opposite_order_lst) in opposite_limit_orders.iter_mut() {
                let is_fully_filled = OrderBook::fill_on_opposite_limit_orders_lst(order, *price, opposite_order_lst, orders);
                if is_fully_filled {
                    break;
                }
            }
            if ! order.is_fully_filled() {
                market_orders.push_back(order.id);
            }
            return order.get_status_based_on_fill();
        }
    }
}

struct MatchingEngine {
    // map from symbol to orderbook
    // order_queue: Vec<Order>
    order_books: HashMap<String, OrderBook>,
    order_id_to_order_book: HashMap<u32, OrderBook> 
}

impl MatchingEngine {
    // FIXME: should matching engine be a static class, or should it have its own instances?
    fn new() -> MatchingEngine {
        MatchingEngine {
            order_books: HashMap::new(),
            order_id_to_order_book: HashMap::new()
        } 
    }

    // TODO: how should this method be structured? Should all order types be handled in one?
    fn process_order(&mut self, order: Order) -> Result<OrderStatus,&'static str> {
        // market orders are executed immediately if possible, otherwise added to queue
        // limit orders are added to queue and executed when the price is reached and its turn comes in queue
        // TODO: how to implement stop orders?
        match self.order_books.get_mut(order.symbol.ticker()) {
            Some(order_book) => Ok(order_book.order(&order)),
            None => Err("symbol does not exist")
        }
    }

}

pub fn process_orders(recv: Receiver<OrderInfo>) {
    // let order_book = self.order_books.get(order.symbol);
    let matching_engine: MatchingEngine = MatchingEngine::new();
    loop {
        // TODO:
    }
}