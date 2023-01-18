#![no_std]
use admin::{check_admin, has_administrator, write_administrator};
use soroban_sdk::{contractimpl, contracttype, map, BytesN, Env, Map};

mod admin;
use token::{Identifier, Signature};
mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_spec.wasm");
}

#[contracttype]
pub enum DataKey {
    Admin,
    Depositors,
    Token,
    Attended, //Store the number of attendees seen
}

#[derive(Clone)]
#[contracttype]
pub struct Depositor {
    pub deposit: i128,
    pub attended: bool,
}

fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract())
}

fn get_attended(e: &Env) -> u32 {
    e.storage()
        .get(DataKey::Attended)
        .expect("no attendees")
        .unwrap()
}

fn increment_attended(e: &Env) {
    let key = DataKey::Attended;
    let attended: u32 = e.storage().get(&key).unwrap_or(Ok(0)).unwrap();
    e.storage().set(key, attended.checked_add(1).unwrap());
}

fn get_token(e: &Env) -> BytesN<32> {
    e.storage()
        .get(DataKey::Token)
        .expect("not initialized")
        .unwrap()
}

fn get_depositors(e: &Env) -> Map<Identifier, Depositor> {
    e.storage()
        .get(DataKey::Depositors)
        .unwrap_or(Ok(map![&e]))
        .unwrap()
}

pub trait DistributionTrait {
    fn initialize(e: Env, admin: Identifier, token: BytesN<32>);
    fn deposit(e: Env, user: Identifier, amount: i128);
    fn attended(e: Env, user: Identifier);
    fn distribute(e: Env);
    fn refund(e: Env);
}

pub struct Distribution;

#[contractimpl]
impl DistributionTrait for Distribution {
    fn initialize(e: Env, admin: Identifier, token: BytesN<32>) {
        if has_administrator(&e) {
            panic!("already initialized")
        }
        write_administrator(&e, admin);

        e.storage().set(DataKey::Token, token);
    }

    fn deposit(e: Env, user: Identifier, amount: i128) {
        //Note: This Map could grow quite large, which would make updates inefficient
        let mut depositors: Map<Identifier, Depositor> = get_depositors(&e);
        if depositors.contains_key(user.clone()) {
            panic!("deposit already set for user");
        }

        // Keep track of user deposits in case we need to refund
        depositors.set(
            user.clone(),
            Depositor {
                deposit: amount,
                attended: false,
            },
        );
        e.storage().set(DataKey::Depositors, depositors);

        let client = token::Client::new(&e, get_token(&e));
        client.xfer_from(
            &Signature::Invoker,
            &0,
            &user,
            &get_contract_id(&e),
            &amount,
        );
    }

    fn attended(e: Env, user: Identifier) {
        check_admin(&e);

        let mut depositors: Map<Identifier, Depositor> = get_depositors(&e);
        if !depositors.contains_key(user.clone()) {
            panic!("deposit missing for user");
        }
        let mut deposit = depositors
            .get_unchecked(user.clone())
            .expect("depositor doesn't exist");
        deposit.attended = true;

        depositors.set(user, deposit);
        e.storage().set(DataKey::Depositors, depositors);

        increment_attended(&e);
    }

    fn distribute(e: Env) {
        // Use different states instead
        check_admin(&e);

        // check admin and state
        let token_client = token::Client::new(&e, get_token(&e));
        let balance = token_client.balance(&get_contract_id(&e));

        // TODO: The remainder will be left in the contract.
        let distribution_amount = balance.checked_div(get_attended(&e) as i128).unwrap();

        let payees: Map<Identifier, Depositor> =
            e.storage().get_unchecked(DataKey::Depositors).unwrap();
        for payee_res in payees {
            let payee = payee_res.unwrap();
            if payee.1.attended {
                token_client.xfer(&Signature::Invoker, &0, &payee.0, &distribution_amount);
            }
        }
    }

    fn refund(e: Env) {
        // Use different states instead
        check_admin(&e);
        let token_client = token::Client::new(&e, get_token(&e));
        let payees: Map<Identifier, Depositor> =
            e.storage().get_unchecked(DataKey::Depositors).unwrap();
        for payee_res in payees {
            let payee = payee_res.unwrap();
            token_client.xfer(&Signature::Invoker, &0, &payee.0, &payee.1.deposit);
        }
    }
}

mod test;
