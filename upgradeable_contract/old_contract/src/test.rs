#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

mod old_contract {
    soroban_sdk::contractimport!(
        file =
            "target/wasm32-unknown-unknown/release/soroban_upgradeable_contract_old_contract.wasm"
    );
}

fn install_new_wasm(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../new_contract/target/wasm32-unknown-unknown/release/soroban_upgradeable_contract_new_contract.wasm"
    );
    e.install_contract_wasm(WASM)
}

#[test]
fn test() {
    let env = Env::default();

    // Note that we use register_contract_wasm instead of register_contract
    // because the old contracts WASM is expected to exist in storage.
    let contract_id = env.register_contract_wasm(None, old_contract::WASM);

    let client = old_contract::Client::new(&env, &contract_id);
    let admin = Address::random(&env);
    client.init(&admin);

    assert_eq!(1, client.version());

    let new_wasm_hash = install_new_wasm(&env);

    client.upgrade(&new_wasm_hash);
    assert_eq!(2, client.version());
}
