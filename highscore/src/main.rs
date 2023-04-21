#![no_main] // not to use the standard main function as its entry point
#![no_std] // not to import the standard libraries

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use core::convert::TryInto;
use alloc::collections::BTreeMap;

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};

use casper_types::{
    api_error::ApiError,
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    CLType, CLValue, Key, URef, Parameter, 
};

const HIGHSCORE_KEY: &str = "highscore_score";
const HIGHSCORE_USER_KEY: &str = "highscore_user_score";

// entry points
const HIGHSCORE_SET: &str = "highscore_set";
const HIGHSCORE_GET: &str = "highscore_get";

const CONTRACT_KEY: &str = "highscore";

#[no_mangle] // ensure that the system does not change critical syntax within the method names
pub extern "C" fn highscore_set() {
    // input parameters
    let key: String = runtime::get_named_arg("name");
    let value: i32  = runtime::get_named_arg("value");

    let score_user: URef = runtime::get_key(HIGHSCORE_KEY)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);

    let hightest_score: i32 = storage::read(score_user)
        .unwrap_or_revert_with(ApiError::Read)
        .unwrap_or_revert_with(ApiError::ValueNotFound);

    // check score with already existed high score
    if value > hightest_score {
        storage::write(score_usef, value);

        let user_uref: URef = runtime::get_key(HIGHSCORE_USER_KEY)
            .unwrap_or_revert_with(ApiError::MissingKey)
            .intro_uref()
            .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);

        storage::write(user_uref, key.as_str());
    }

    // check is this score already presented foe this user
    match runtime::get_key(key.as_str()) {
        Some(key) => {
            // user has played before
            let key_ref = key.try_info().unwrap_or_revert();
            let users_highest_score: i32 = storage::read(key_ref)
                .unwrap_or_revert_with(ApiError::Read)
                .unwrap_or_revert_with(ApiError::ValueNotFound);

            // rewrite the highest score for user
            if value > users_highest_score {
                storage::write(key_ref, value);
            }
        }

        None => {
            let value_ref = storage::new_uref(value);
            let value_key = Key::URef(value_ref);
            runtime::put_key(key.as_str(), value_key);
        }
    }
}

#[no_mangle]
pub extern "C" fn highscore_get() {
    let name: String = runtime::get_named_arg("name");

    let user: URef = runtime::get_key(&name)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);
        
    let result: i32 = storage::read(uref)
        .unwrap_or_revert_with(ApiError::Read)
        .unwrap_or_revert_with(ApiError::ValueNotFound);

    runtime::ret(CLValue::from_t(result).unwrap_or_revert());
}

#[no_mangle] 
// main function
pub extern "C" fn call() {
    let highscore_local_key = storage::new_uref(0_i32);

    let highscore_user_key = storage::new_uref("");

    let mut highscore_named_keys: BTreeMap<String, Key> = BTreeMap::new();
    let key_name = String::from(HIGHSCORE_KEY);
    highscore_named_keys.insert(key_name, highscore_local_key.into());
    let key_user_name = String::from(HIGHSCORE_USER_KEY);
    highscore_named_keys.insert(key_user_name, highscore_user_key.into());

    let mut highscore_entry_points = EntryPoints::new();
    highscore_entry_points.add_entry_point(EntryPoint::new (
        HIGHSCORE_SET,
        vec![
            Parameter::new("name", CLType::String),
            Parameter::new("value", CLType::i32)
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointAccess::Contract,
    ));

    highscore_entry_points.add_entry_point(EntryPoint::new (
        HIGHSCORE_GET,
        vec![
            Parameter::new("name", CLType::String)
        ],
        CLType::String,
        EntryPointAccess::Public,
        EntryPointAccess::Contract,
    ));

    let (stored_contract_hash, _) = storage::new_locked_contract(highscore_entry_points, Some(highscore_named_keys), None, None);
    runtime::put_key(CONTRACT_KEY, stored_contract_hash.into());

}
