use std::collections::HashMap;

/// a struct tracking all currently active accounts
struct Accountant {
    accounts: HashMap<u32, Account>
}

impl Accountant {
    /// creates a new Accountant and initializes the account map
    pub fn new() -> Accountant {
        Accountant {
            accounts: HashMap<u32, Account>::new()
        }
    }

    /// registers an account with the Accountant
    /// 
    /// # Parameters
    /// 
    /// * `account` - the account to register
    pub fn register(&mut self, account: Account) {
        // FIXME: does self need to be mutable?
        self.accounts.insert(account.id, account);
    }
}