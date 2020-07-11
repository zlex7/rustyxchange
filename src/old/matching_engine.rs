use std::cmp;
use std::error::Error;
use std::collections::BTreeMap;
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{Receiver, Sender};
use std::fmt;

use super::SYMBOLS;
use types::*;

#[derive(Debug, Clone)]
struct InvalidOrderId;

impl fmt::Display for InvalidOrderId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid order id given")
    }
}

impl Error for InvalidOrderId {
    fn description(&self) -> &str {
        "invalid order id given"
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug, Clone)]
struct InvalidTicker;

impl fmt::Display for InvalidTicker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid order id given")
    }
}

impl Error for InvalidTicker {
    fn description(&self) -> &str {
        "invalid order id given"
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug, Clone)]
struct EmptyOrderBook;

impl fmt::Display for EmptyOrderBook {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "no orders in orderbook")
    }
}

impl Error for EmptyOrderBook {
    fn description(&self) -> &str {
        "no orders in orderbook"
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
/// a struct containing a list of open bids and asks
struct OrderBook {
    symbol: &'static Symbol,
    pub bids: BTreeMap<u64, VecDeque<u32>>,
    pub asks: BTreeMap<u64, VecDeque<u32>>,
    pub market_bids: VecDeque<u32>,
    pub market_asks: VecDeque<u32>,
    orders: HashMap<u32, Order>,
}

impl OrderBook {
    fn new(symbol: &'static Symbol) -> OrderBook {
        OrderBook {
            symbol: symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            market_bids: VecDeque::new(),
            market_asks: VecDeque::new(),
            orders: HashMap::new(),
        }
    }

    // order priority:
    //  1. market > limit
    //  2. timestamp

    pub fn print_book(&self) -> () {
        let ticker = self.symbol.ticker();
        println!("{} new limit order bids: {:?}", ticker, self.bids);
        println!("{} new limit order asks: {:?}", ticker, self.asks);
        println!("{} new market order bids: {:?}", ticker, self.market_bids);
        println!("{} new market order asks: {:?}", ticker, self.market_asks);

        println!("{} new limit order bids:", ticker);
        for (price, order_lst) in self.bids.iter() {
            println!("price: {}", &price);
            for order_id in order_lst.iter() {  
                println!("order: {:?}", self.orders.get(&order_id).unwrap());
            }
        }
        println!("{} new limit order asks: ", ticker);
        for (price, order_lst) in self.asks.iter() {
            println!("price: {}", &price);
            for order_id in order_lst.iter() {  
                println!("order: {:?}", self.orders.get(&order_id).unwrap());
            }
        }
        println!("{} new market order bids: ", ticker);
        for order_id in self.market_bids.iter() {
            println!("order: {:?}", self.orders.get(&order_id).unwrap());
        }
        println!("{} new market order asks: ", ticker);
        for order_id in self.market_asks.iter() {
            println!("order: {:?}", self.orders.get(&order_id).unwrap());
        }
    }

    pub fn status(&self, order_id: u32) -> Result<OrderStatus, Box<dyn Error>> {
        let order = self.orders.get(&order_id).ok_or(InvalidOrderId)?;
        Ok (order.get_status_based_on_fill())
    }

    fn delete_empty_price_levels_generic(map: &mut BTreeMap<u64,VecDeque<u32>>) {
        let mut prices_to_delete = Vec::new();
        for (price, order_lst) in map.iter() {
            if order_lst.len() == 0 {
                prices_to_delete.push(price.clone());
            }
        };
        for price in prices_to_delete {
            map.remove(&price);
        }
    }

    pub fn delete_empty_price_levels(&mut self) {
        OrderBook::delete_empty_price_levels_generic(&mut self.bids);
        OrderBook::delete_empty_price_levels_generic(&mut self.asks);
    }

    fn delete_bid_price_level(&mut self, price: u64) {
        if self.bids.get(&price).expect(&format!("bids doesn't contain price = {}", price)).len() == 0 {
            self.bids.remove(&price);
        }
    }

    fn delete_ask_price_level(&mut self, price: u64) {
        if self.asks.get(&price).expect(&format!("asks doesn't contain price = {}", price)).len() == 0 {
            self.asks.remove(&price);
        }
    }

    fn remove_order(&mut self, order_id: u32) -> () {
        let order = self.orders.get(&order_id).expect("invalid order id in remove()");
        match order.order_type {
            OrderType::Limit(price) => {
                match order.side {
                    OrderSide::Buy => {
                        self.bids
                            .get_mut(&price)
                            .unwrap()
                            .retain(|x| *x != order_id);
                        self.delete_bid_price_level(price);
                    }
                    OrderSide::Sell => {
                        self.asks
                            .get_mut(&price)
                            .unwrap()
                            .retain(|x| *x != order_id);
                        self.delete_ask_price_level(price);
                    }
                };
            }
            OrderType::Market => match order.side {
                OrderSide::Buy => {
                    self.market_bids.retain(|x| *x != order_id);
                }
                OrderSide::Sell => {
                    self.market_asks.retain(|x| *x != order_id);
                }
            },
            _ => {}
        };
        return ();
    }

    fn print_orders(&self) {
        println!("orders = {:?}", self.orders);
    }

    pub fn cancel(&mut self, order_id: u32) -> Result<OrderStatus, Box<dyn Error>> {
        self.print_orders();
        let order = self.orders.get_mut(&order_id).ok_or(InvalidOrderId)?;
        println!("cancelling order = {:?}", order);
        if order.is_canceled || order.remaining_quantity == 0 {
            return self.status(order_id);
        }
        order.is_canceled = true;
        let order = self.orders.get(&order_id).ok_or(InvalidOrderId)?;
        self.remove_order(order.id);        

        self.delete_empty_price_levels();
        self.print_book();
        return self.status(order_id)
    }

    fn get_top_level(&self) -> (u64, u64, u64, u64) {
        let (best_bid, best_bid_size) : (u64, u64) = match self.bids.len() != 0 {
            true => {
                let (bid, bid_list) = self.bids.iter().rev().next().unwrap();
                (*bid, bid_list
            .iter()
            .map(|o| self.orders.get(o).unwrap().remaining_quantity)
            .sum())
            },
            false => (0, 0)
        };

        let (best_ask, best_ask_size) : (u64, u64) = match self.asks.len() != 0 {
            true => {
                let (ask, ask_list) = self.asks.iter().next().unwrap();
                println!("ask list: {:?}", ask_list);
                println!("orders: {:?}", self.orders);
                (*ask, ask_list
            .iter()
            .map(|o| self.orders.get(o).unwrap().remaining_quantity)
            .sum())
            },
            false => (0 as u64, 0 as u64)
        };

        return (best_bid, best_bid_size, best_ask, best_ask_size);
    }
    //TODO: one problem we need to deal with is making appropiate variables mutable in Order struct
    pub fn order(&mut self, old_order: &Order, send: Sender<PriceInfo>) -> Result<OrderStatus, Box<dyn Error>> {
        // self.orders.insert(old_order.id, old_order.clone());
        // let order : &mut Order = self.orders.get_mut(&old_order.id).unwrap();
        let mut order = old_order.clone();
        let (best_bid, best_bid_size, best_ask, best_ask_size) = self.get_top_level();  

        println!("filling order...");
        let order_status = match order.order_type {
            OrderType::Market => self.market_order(&mut order),
            OrderType::Limit(price) => self.limit_order(&mut order, price),
            OrderType::Stop(price) => self.stop_order(&mut order, price),
        };
        println!("inserting order id = {}", order.id);
        self.orders.insert(order.id, order);

        println!("done filling order!");

        self.delete_empty_price_levels();

        let (new_best_bid, new_best_bid_size, new_best_ask, new_best_ask_size) = self.get_top_level();  

        if new_best_bid > best_bid
            || new_best_ask < best_ask
            || new_best_bid_size != best_bid_size
            || new_best_ask_size != best_ask_size
        {
            send.send(PriceInfo::new(
                self.symbol,
                new_best_bid,
                new_best_bid_size,
                new_best_ask,
                new_best_ask_size,
            )).expect("[ERROR] failed to send price info to market data server");
        }

        return Ok(order_status)
    }

    pub fn stop_order(&mut self, order: &mut Order, price: u64) -> OrderStatus {
        return OrderStatus::Waiting(order.id);
    }

    pub fn limit_order(&mut self, order: &mut Order, price: u64) -> OrderStatus {
        match order.side {
            OrderSide::Buy => {
                OrderBook::limit_order_generic(
                    order,
                    price,
                    &mut self.bids,
                    &mut self.asks,
                    &mut self.market_bids,
                    &mut self.orders,
                )
            }
            OrderSide::Sell => {
                OrderBook::limit_order_generic(
                    order,
                    price,
                    &mut self.asks,
                    &mut self.bids,
                    &mut self.market_asks,
                    &mut self.orders,
                )
            }
        }
    }

    pub fn market_order(&mut self, order: &mut Order) -> OrderStatus {
        match order.side {
            OrderSide::Buy => {
                return OrderBook::market_order_generic(
                    order,
                    &mut self.asks,
                    &mut self.market_bids,
                    &mut self.orders,
                );
            }
            OrderSide::Sell => {
                return OrderBook::market_order_generic(
                    order,
                    &mut self.bids,
                    &mut self.market_asks,
                    &mut self.orders,
                );
            }
        }
    }

    fn fill_on_opposite_limit_orders_lst(
        order: &mut Order,
        price: u64,
        opposite_order_lst: &mut VecDeque<u32>,
        orders: &mut HashMap<u32, Order>,
    ) -> bool {
        let mut num_orders_to_remove = 0;
        for id in opposite_order_lst.iter_mut() {
            let opposite_order: &mut Order = orders.get_mut(id).unwrap();
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
        for _ in 0..num_orders_to_remove {
            opposite_order_lst.pop_front();
        }
        return order.is_fully_filled();
    }

    fn list_limit_order(
        order: &mut Order,
        price_per_share: u64,
        limit_orders: &mut BTreeMap<u64, VecDeque<u32>>,
    ) -> () {
        limit_orders
            .entry(price_per_share)
            .or_insert(VecDeque::new());
        limit_orders
            .get_mut(&price_per_share)
            .unwrap()
            .push_back(order.id);
    }

    fn limit_order_generic(
        order: &mut Order,
        price_per_share: u64,
        same_side_limit_orders: &mut BTreeMap<u64, VecDeque<u32>>,
        opposite_limit_orders: &mut BTreeMap<u64, VecDeque<u32>>,
        market_orders: &mut VecDeque<u32>,
        orders: &mut HashMap<u32, Order>,
    ) -> OrderStatus {
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
        for _ in 0..num_to_remove {
            market_orders.pop_front();
        }
        if order.is_fully_filled() {
            return order.get_status_based_on_fill();
        }

        for (opposite_price, opposite_order_lst) in opposite_limit_orders.iter_mut() {
            if (order.side == OrderSide::Buy && *opposite_price > price_per_share)
                || (order.side == OrderSide::Sell && *opposite_price < price_per_share)
            {
                break;
            }
            let is_fully_filled = OrderBook::fill_on_opposite_limit_orders_lst(
                order,
                price_per_share,
                opposite_order_lst,
                orders,
            );
            if is_fully_filled {
                break;
            }
        }

        if !order.is_fully_filled() {
            OrderBook::list_limit_order(order, price_per_share, same_side_limit_orders);
        }

        order.get_status_based_on_fill()
    }

    //TODO: remove key from BTree when no orders left at that price.
    fn market_order_generic(
        order: &mut Order,
        opposite_limit_orders: &mut BTreeMap<u64, VecDeque<u32>>,
        market_orders: &mut VecDeque<u32>,
        orders: &mut HashMap<u32, Order>,
    ) -> OrderStatus {
        if opposite_limit_orders.len() == 0 {
            market_orders.push_back(order.id);
            return OrderStatus::Waiting(order.id);
        } else {
            for (price, opposite_order_lst) in opposite_limit_orders.iter_mut() {
                let is_fully_filled = OrderBook::fill_on_opposite_limit_orders_lst(
                    order,
                    *price,
                    opposite_order_lst,
                    orders,
                );
                if is_fully_filled {
                    break;
                }
            }
            if !order.is_fully_filled() {
                market_orders.push_back(order.id);
            }
            return order.get_status_based_on_fill();
        }
    }
}

struct MatchingEngine {
    pub order_books: HashMap<&'static str, OrderBook>,
    order_id_to_symbol: HashMap<u32, &'static Symbol>,
    market_data_send: Sender<PriceInfo>,
}

impl MatchingEngine {
    fn new(market_data_send: Sender<PriceInfo>) -> MatchingEngine {
        let mut order_books: HashMap<&str, OrderBook> = HashMap::new();
        for (ticker, symbol) in SYMBOLS.iter() {
            println!("saving {:?} in order books", symbol);
            order_books.insert(ticker, OrderBook::new(symbol));
        }

        let m_engine = MatchingEngine {
            order_books: order_books,
            order_id_to_symbol: HashMap::new(),
            market_data_send: market_data_send,
        };
        return m_engine;
    }

    fn process_order(&mut self, order: Order) -> Result<OrderStatus, Box<dyn Error>> {
        // market orders are executed immediately if possible, otherwise added to queue
        // limit orders are added to queue and executed when the price is reached and its turn comes in queue
        // TODO: how to implement stop orders?
        println!("processing symbol for {:?}", order);
        let ret = match self.order_books.get_mut(order.symbol.ticker()) {
            Some(order_book) => {
                println!("inserting order {:?} into order book for {:?}", order, order.symbol);
                Ok(order_book.order(&order, self.market_data_send.clone()))
            },
            None => Err("symbol does not exist in process_order()"),
        };
        match ret {
            Ok(_) => {
                self.order_id_to_symbol
                    .insert(order.id, order.symbol);
            }
            _ => {}
        }

        return ret.unwrap();
    }

    fn status(&self, order_id: u32) -> Result<OrderStatus, Box<dyn Error>> {
        let ticker = self.order_id_to_symbol.get(&order_id).ok_or(InvalidOrderId)?.ticker();
        let order_book = self.order_books.get(ticker).ok_or(InvalidTicker)?;
        order_book.status(order_id)
    }

    fn cancel(&mut self, order_id: u32) -> Result<OrderStatus, Box<dyn Error>> {
        let ticker = self.order_id_to_symbol.get(&order_id).ok_or(InvalidOrderId)?.ticker();
        let order_book = self.order_books.get_mut(ticker).ok_or(InvalidTicker)?;
        order_book.cancel(order_id)
    }
}

pub fn process_orders(market_data_send: Sender<PriceInfo>, recv: Receiver<Cmd>) {
    // let order_book = self.order_books.get(order.symbol);
    let mut matching_engine: MatchingEngine = MatchingEngine::new(market_data_send.clone());
    let mut ORDER_ID_COUNTER: u32 = 0 as u32;
    // TODO: handle errors
    loop {
        // let order_info = recv.recv();
        // matching_engine.process_order(Order {})
        for _ in 0..1000 {
            if let Ok(cmd) = recv.try_recv() {
                match cmd {
                    Cmd::Execute(order_info) => {
                        let (order, sender) = order_info.consume(ORDER_ID_COUNTER);
                        ORDER_ID_COUNTER += 1;

                        let ticker = order.symbol.ticker().clone();
                        let status = matching_engine.process_order(order).unwrap();
                        sender
                            .send(status)
                            .expect("[ERROR]: EXECUTE failed to send client status to client");
                        matching_engine.order_books.get(ticker).unwrap().print_book();
                    }
                    Cmd::Status(status_info) => {
                        let (account_id, order_id, sender) = status_info.consume();
                        // TODO: need method for getting status
                        let status = matching_engine.status(order_id).expect("[ERROR] failed to get status of order");
                        sender
                            .send(status)
                            .expect("[ERROR]: STATUS failed to send client status to client");
                    }
                    Cmd::Cancel(cancel_info) => {
                        let (account_id, order_id, sender) = cancel_info.consume();
                        // TODO: need method for canceling
                        let status = matching_engine.cancel(order_id).unwrap();
                        sender
                            .send(status)
                            .expect("[ERROR]: CANCEL failed to send client status to client");
                    }
                };
            }
        }
    }
}