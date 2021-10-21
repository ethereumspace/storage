use context::{metadata, util};
use ic_cdk::export::candid::Nat;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::print;
use ic_cdk::storage;
use ic_cdk_macros::*;
use std::collections::HashMap;
type CanisterTransaction = HashMap<String, Vec<usize>>;
type CallerTransaction = HashMap<String, Vec<usize>>;
static mut Transaction: Vec<metadata::Metadata> = vec![];

#[derive(CandidType, Deserialize)]
struct Snapshot<T> {
    key: String,
    value: T,
}

#[derive(CandidType, Deserialize)]
struct Db {
    canisterTransaction: Vec<Snapshot<Vec<usize>>>,
    callerTransaction: Vec<Snapshot<Vec<usize>>>,
    transacton: Vec<metadata::Metadata>,
}

/// Add transaction
#[update(name = createTransaction)]
fn create_transaction(metadata: metadata::Metadata) -> Result<(), String> {
    let position: usize;
    unsafe {
        Transaction.push(metadata.clone());
        position = Transaction.len() - 1;
    }
    let canister_transaction = storage::get_mut::<CanisterTransaction>();
    if !canister_transaction.contains_key(&metadata.canister) {
        canister_transaction.insert(metadata.canister.clone(), vec![position]);
    }
    let caller_transaction = storage::get_mut::<CallerTransaction>();
    if !caller_transaction.contains_key(&metadata.caller) {
        caller_transaction.insert(metadata.caller.clone(), vec![position]);
        return Ok(());
    }

    canister_transaction
        .get_mut(&metadata.canister)
        .unwrap()
        .push(position);
    caller_transaction
        .get_mut(&metadata.caller)
        .unwrap()
        .push(position);

    let len = canister_transaction.get(&metadata.canister).unwrap().len();
    let info = format!("count {}", len);
    print(info);
    Ok(())
}

#[query(name = "getCanisterTransaction")]
async fn get_canister_transaction(
    canister: String,
    offset: Nat,
    limit: Nat,
) -> Vec<metadata::Metadata> {
    let offset = util::nat_to_u64(offset).unwrap() as usize;
    let mut limit = util::nat_to_u64(limit).unwrap() as usize;
    if limit > 50 {
        limit = 50;
    }
    let canister_transaction = storage::get::<CanisterTransaction>();
    if !canister_transaction.contains_key(&canister) {
        print("canister not exist");
        return vec![];
    }
    let blucket = canister_transaction.get(&canister).unwrap();
    let info = format!("count {},{},{}", blucket.len(), offset, limit);
    if offset > blucket.len() {
        return vec![];
    }
    let mut container: Vec<metadata::Metadata> = vec![];
    unsafe {
        if offset + limit > blucket.len() {
            for i in blucket[offset..info.len()].iter() {
                container.push(Transaction[*i].clone());
            }
            return container;
        }

        for i in blucket[offset..offset + limit].iter() {
            container.push(Transaction[*i].clone());
        }
        return container;
    }
}

/// Get caller transaction transaction record information
#[query(name = "getCallerTransaction")]
async fn get_caller_transaction(
    caller: String,
    offset: Nat,
    limit: Nat,
) -> Vec<metadata::Metadata> {
    let offset = util::nat_to_u64(offset).unwrap() as usize;
    let mut limit = util::nat_to_u64(limit).unwrap() as usize;
    if limit > 50 {
        limit = 50;
    }
    let caller_transaction = storage::get::<CallerTransaction>();
    if !caller_transaction.contains_key(&caller) {
        print("canister not exist");
        return vec![];
    }
    let blucket = caller_transaction.get(&caller).unwrap();
    let info = format!("count {},{},{}", blucket.len(), offset, limit);
    if offset > blucket.len() {
        return vec![];
    }
    let mut container: Vec<metadata::Metadata> = vec![];
    unsafe {
        if offset + limit > blucket.len() {
            for i in blucket[offset..info.len()].iter() {
                container.push(Transaction[*i].clone());
            }
            return container;
        }

        for i in blucket[offset..offset + limit].iter() {
            container.push(Transaction[*i].clone());
        }
        return container;
    }
}

#[query(name = "getLastTransaction")]
async fn get_last_transaction(limit: Nat) -> Vec<metadata::Metadata> {
    let limit = util::nat_to_u64(limit).unwrap() as usize;
    unsafe {
        let len = Transaction.len();
        if len <= limit {
            return Transaction.to_vec();
        }
        return Transaction[len - limit - 1..len - 1].to_vec();
    }
}

#[query(name = "getCanisterLastTransaction")]
async fn get_canister_last_transaction(canister: String, limit: Nat) -> Vec<metadata::Metadata> {
    let limit = util::nat_to_u64(limit).unwrap() as usize;
    let canister_transaction = storage::get::<CanisterTransaction>();
    let transaction = canister_transaction.get(&canister).unwrap();
    let len = transaction.len();
    let mut result: Vec<metadata::Metadata> = vec![];
    unsafe {
        if len <= limit {
            for i in transaction.iter() {
                result.push(Transaction[*i].clone())
            }
            return result;
        }
        for j in transaction[len - limit - 1..len - 1].iter() {
            result.push(Transaction[*j].clone())
        }
        return result;
    }
}

/// Before the upgrade task starts, you need to persist the data in memory
#[pre_upgrade]
fn pre_upgrade() {
    let mut canister_transction_snapshot: Vec<Snapshot<Vec<usize>>> = vec![];
    let mut caller_transction_snapshot: Vec<Snapshot<Vec<usize>>> = vec![];
    let canister_transction = storage::get::<CanisterTransaction>();
    for (k, v) in canister_transction.iter() {
        let snapshot = Snapshot::<Vec<usize>> {
            key: k.clone(),
            value: v.to_vec(),
        };
        canister_transction_snapshot.push(snapshot);
    }
    let caller_transction = storage::get::<CallerTransaction>();

    for (k, v) in caller_transction.iter() {
        let snapshot = Snapshot::<Vec<usize>> {
            key: k.clone(),
            value: v.to_vec(),
        };
        caller_transction_snapshot.push(snapshot);
    }

    unsafe {
        let db = Db {
            canisterTransaction: canister_transction_snapshot,
            callerTransaction: caller_transction_snapshot,
            transacton: Transaction.to_vec(),
        };
        storage::stable_save((db,));
    }
    let size = ic_cdk::api::stable::stable_size();
    let size = format!("当前mem page 大小: {}", size);
    print(size);
}

/// Before the upgrade task ends, you need to reload the persistent data into memory
#[post_upgrade]
fn post_update() {
    let db = storage::stable_restore::<(Db,)>().expect("failed to stable_restore");
    data_load(db.0);
    print("升级成功");
}

/// Reload data into memory
fn data_load(db: Db) {
    unsafe {
        Transaction = db.transacton;
    }
    let canister_transaction = storage::get_mut::<CanisterTransaction>();
    for i in db.canisterTransaction.into_iter() {
        canister_transaction.insert(i.key.clone(), i.value);
    }
    for j in db.callerTransaction.into_iter() {
        canister_transaction.insert(j.key.clone(), j.value);
    }
}
