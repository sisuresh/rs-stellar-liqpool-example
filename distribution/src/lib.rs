#![no_std]
use soroban_sdk::{contractimpl, contracttype, BytesN, Env, Set};

use token::{Identifier, Signature};
mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

/// Requirements
///
/// Profit distribution - when asset is deposited, split it up between stakeholders
///
/// 1. Equal split vs % stake
/// 2. How to make this automatic? I think we need a trigger function that takes
///    a token address and then pays out? Trigger function could
///    also take an int to limit the number of xfers in a single invocation.
/// 3. Trigger method to pay out. Should we also include a method that takes an amount
///    and calls xfer_from?
/// 4. Admin that initializes and controls members of profit pool
/// 5. Store a vec of Identifiers?
/// 6. Will need to understand how to use the token interface!!!

#[contracttype]
pub enum DataKey {
    Payees,
}

fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract())
}

//TODO: Add admin!!!
pub trait DistributionTrait {
    fn initialize(e: Env, payees: Set<Identifier>);
    //fn remove(e: Env, payee: Identifier);
    //fn add(e: Env, payee: Identifier);
    fn distribute(e: Env, token: BytesN<32>);
}

pub struct Distribution;

#[contractimpl]
impl DistributionTrait for Distribution {
    fn initialize(e: Env, payees: Set<Identifier>) {
        e.storage().set(DataKey::Payees, payees);
    }

    fn distribute(e: Env, token: BytesN<32>) {
        let token_client = token::Client::new(&e, token);
        let balance = token_client.balance(&get_contract_id(&e));

        let payees: Set<Identifier> = e.storage().get_unchecked(DataKey::Payees).unwrap();

        // The remainder will be left in the contract, and can be claimed in the future once
        // the balance increases.
        let distribution_amount = balance.checked_div(payees.len() as i128).unwrap();

        for payee in payees {
            token_client.xfer(
                &Signature::Invoker,
                &0,
                &payee.unwrap(),
                &distribution_amount,
            );
        }
    }
}

mod test;
