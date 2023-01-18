#![cfg(test)]

use super::*;

use soroban_sdk::{
    testutils::{Accounts, Ledger, LedgerInfo},
    AccountId, Env, IntoVal,
};

mod token {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );
}

fn create_distribution_contract(
    e: &Env,
    admin: &AccountId,
    token: &BytesN<32>,
) -> DistributionClient {
    let contract_id = e.register_contract(None, Distribution);
    let client = DistributionClient::new(&e, &contract_id);
    client.initialize(&soroban_auth::Identifier::Account(admin.clone()), &token);
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

struct DistributionTest {
    admin: AccountId,
    users: [AccountId; 3],
    token: token::Client,
    contract: DistributionClient,
    contract_id: Identifier,
}

impl DistributionTest {
    fn setup() -> Self {
        let env: Env = Default::default();
        env.ledger().set(LedgerInfo {
            timestamp: 12345,
            protocol_version: 1,
            sequence_number: 10,
            network_passphrase: Default::default(),
            base_reserve: 10,
        });

        let users = [
            env.accounts().generate(),
            env.accounts().generate(),
            env.accounts().generate(),
        ];

        let token_admin = env.accounts().generate();
        let token = create_token_contract(&env, &token_admin);

        let admin = env.accounts().generate();
        let contract = create_distribution_contract(&env, &admin, &token.contract_id);
        let contract_id = Identifier::Contract(contract.contract_id.clone());

        for attendee in users.clone() {
            token.with_source_account(&token_admin).mint(
                &Signature::Invoker,
                &0,
                &Identifier::Account(attendee.clone()),
                &1000,
            );
            token.with_source_account(&attendee).incr_allow(
                &Signature::Invoker,
                &0,
                &contract_id,
                &1000,
            )
        }

        DistributionTest {
            admin,
            users,
            token,
            contract,
            contract_id,
        }
    }
}

#[test]
fn test_distribute() {
    let test = DistributionTest::setup();

    let user1_id = Identifier::Account(test.users[0].clone());
    let user2_id = Identifier::Account(test.users[1].clone());
    let user3_id = Identifier::Account(test.users[2].clone());

    test.contract.deposit(&user1_id, &1000);
    test.contract.deposit(&user2_id, &1000);
    test.contract.deposit(&user3_id, &1000);

    assert_eq!(test.token.balance(&test.contract_id), 3000);
    assert_eq!(test.token.balance(&user1_id), 0);
    assert_eq!(test.token.balance(&user2_id), 0);

    test.contract
        .with_source_account(&test.admin)
        .attended(&user1_id);
    test.contract
        .with_source_account(&test.admin)
        .attended(&user2_id);
    test.contract.with_source_account(&test.admin).distribute();

    assert_eq!(test.token.balance(&test.contract_id), 0);
    assert_eq!(test.token.balance(&user1_id), 1500);
    assert_eq!(test.token.balance(&user2_id), 1500);
    assert_eq!(test.token.balance(&user3_id), 0);
}

#[test]
fn test_refund() {
    let test = DistributionTest::setup();

    let user1_id = Identifier::Account(test.users[0].clone());
    let user2_id = Identifier::Account(test.users[1].clone());
    let user3_id = Identifier::Account(test.users[2].clone());

    test.contract.deposit(&user1_id, &1000);
    test.contract.deposit(&user2_id, &1000);
    test.contract.deposit(&user3_id, &1000);

    assert_eq!(test.token.balance(&test.contract_id), 3000);
    assert_eq!(test.token.balance(&user1_id), 0);
    assert_eq!(test.token.balance(&user2_id), 0);

    test.contract.with_source_account(&test.admin).refund();

    assert_eq!(test.token.balance(&test.contract_id), 0);
    assert_eq!(test.token.balance(&user1_id), 1000);
    assert_eq!(test.token.balance(&user2_id), 1000);
    assert_eq!(test.token.balance(&user3_id), 1000);
}
