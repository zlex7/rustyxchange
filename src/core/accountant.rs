use std::collections::HashMap;

// moved from matching_engine
struct Account {
    positions: HashMap<String, i64>,
    balance: f64,
    id: u32
}

impl Account {
    fn new(id: u32) -> Account {
        Account {
            positions: HashMap<String, i64>::new(),
            balance: 0,
            id: id
        }
    }
}

struct Accountant {
    accounts: HashMap<u32, Account>
}

impl Accountant {
    pub fn new() -> Accountant {
        Accountant {
            accounts: HashMap<u32, Account>::new()
        }
    }

    // FIXME: does self need to be mutable?
    pub fn register(&self, account: Account) {
        self.accounts.insert(account.id, account);
    }
}