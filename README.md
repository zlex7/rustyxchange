# rustyxchange
it's an exchange, and its rusty

# Protocol for Order Sending
The data is structured as an array of bytes (`[u8]`). 
`data[0] & 3` is the command type (in the src, `CmdType` enum). It can take 3 possible values: Execute(0), Status(2), Cancel(3)
_(Note that I'll use Rust-like syntax. Thus, `..` means exclusive range, and so on.)_
Additionally, `data[1..5]` is a 32-bit integer representing the order id.
The above two fields are common to all order types. After, the 3 orders differ in internal structure. I'll go over each one briefly below.

### Execute Order
`Execute` takes 26 bytes to represent.
`data[0] >> 2` is the order side (in src, `OrderSide` enum), which is a single bit representing whether the order is buy-side or sell-side. 0 means buy, 1 means sell.
`data[5]` is the order type (in src, `OrderType` enum). Currently, there are 3 possible values: Market(0), Limit(1), Stop(2). I've also included the definition of the enum below:
```rust
pub enum OrderType {
    Market,
    Limit(u64),
    Stop(u64)
}
```

Note the nested values in the enum (representing the price, this will be important later.
`data[6..10]` is the ticker. Currently, all tickers MUST be 4 bytes long.
`data[10..18]` is an unsigned 64-bit integer representing the stop/limit price, multiplied by 1000.
`data[18..26]` is another unsigned 64-bit integer representing the quantity.

### Status
`Status` takes only 9 bytes to represent.
`data[5..9]` is an unsigned 32-bit integer representing the order id to get the status of.

### Cancel
`Cancel` is exactly the same as `Status`, taking only 9 bytes to represent.
`data[5..9]` is an unsigned 32-bit integer representing the order id to cancel.

# Testing
Once the client side is done, you can clone the exchange repo and run it locally (`cargo run` basically). It should print two IP addresses/ports. Use the one that's marked as gateway i.e. `[INFO] gateway started on 0.0.0.0:8888`. You should connect to this IP through a TCP connection after which you can send the data.
To send data, you must send the size of the data before you send the data itself (i.e. for an execute order, you should first send 26).

_Note: this is subject to change_
