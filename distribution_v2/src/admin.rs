use super::DataKey;
use soroban_auth::Identifier;
use soroban_sdk::{Address, Env};

pub fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.storage().has(key)
}

fn read_administrator(e: &Env) -> Identifier {
    let key = DataKey::Admin;
    e.storage().get_unchecked(key).unwrap()
}

pub fn write_administrator(e: &Env, id: Identifier) {
    let key = DataKey::Admin;
    e.storage().set(key, id);
}

pub fn check_admin(e: &Env) {
    if let Address::Account(account_id) = e.invoker() {
        if Identifier::Account(account_id) == read_administrator(e) {
            return;
        }
    }
    panic!("not authorized by admin");
}
