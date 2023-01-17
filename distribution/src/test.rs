#![cfg(test)]

use super::*;

use soroban_sdk::{set, testutils::Accounts, AccountId, Env, IntoVal};

mod token {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );
}

fn create_distribution_contract(e: &Env, payees: &Set<Identifier>) -> DistributionClient {
    let contract_id = e.register_contract(None, Distribution);
    let client = DistributionClient::new(&e, &contract_id);
    client.initialize(&payees);
    client
}

fn create_token_contract(e: &Env, admin: &AccountId) -> token::Client {
    e.install_contract_wasm(token::WASM);

    let token = token::Client::new(e, e.register_contract_wasm(None, token::WASM));
    // decimals, name, symbol don't matter in tests
    token.initialize(
        &Identifier::Account(admin.clone()),
        &7u32,
        &"name".into_val(e),
        &"symbol".into_val(e),
    );
    token
}

#[test]
fn test() {
    let env = Env::default();

    let admin = env.accounts().generate();
    let user1 = env.accounts().generate();
    let user2 = env.accounts().generate();
    let user1_id = Identifier::Account(user1);
    let user2_id = Identifier::Account(user2);

    let payees = set![&env, user1_id.clone(), user2_id.clone()];

    let distribution_client = create_distribution_contract(&env, &payees);
    let distribution_id = Identifier::Contract(distribution_client.contract_id.clone());

    let token_client = create_token_contract(&env, &admin);

    token_client
        .with_source_account(&admin)
        .mint(&Signature::Invoker, &0, &distribution_id, &100);

    assert_eq!(token_client.balance(&distribution_id), 100);
    assert_eq!(token_client.balance(&user1_id), 0);
    assert_eq!(token_client.balance(&user2_id), 0);

    //let token_id = Identifier::Contract(token_client.contract_id.clone());
    distribution_client.distribute(&token_client.contract_id);

    assert_eq!(token_client.balance(&distribution_id), 0);
    assert_eq!(token_client.balance(&user1_id), 50);
    assert_eq!(token_client.balance(&user2_id), 50);
}
