use context::{metadata, util};
use ic_cdk::export::candid::Nat;
use ic_cdk::export::Principal;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::print;
use ic_cdk::storage;
use ic_cdk_macros::*;
use std::collections::HashMap;
mod config;
type CanisterEvent = HashMap<Principal, Vec<usize>>;
type CallerEvent = HashMap<Principal, Vec<usize>>;
static mut Event: Vec<metadata::Metadata> = vec![];

#[derive(CandidType, Deserialize)]
struct Snapshot<T> {
    key: Principal,
    value: T,
}

#[derive(CandidType, Deserialize)]
struct Db {
    canisterEvent: Vec<Snapshot<Vec<usize>>>,
    callerEvent: Vec<Snapshot<Vec<usize>>>,
    event: Vec<metadata::Metadata>,
}

/// Add transaction
#[update(name = "createEvent")]
fn create_event(metadata: metadata::Metadata) -> Result<(), String> {
    let position: usize;
    unsafe {
        Event.push(metadata.clone());
        position = Event.len() - 1;
    }
    let canister_event = storage::get_mut::<CanisterEvent>();
    if !canister_event.contains_key(&metadata.canister) {
        canister_event.insert(metadata.canister.clone(), vec![position]);
    }
    let caller_event = storage::get_mut::<CallerEvent>();
    if !caller_event.contains_key(&metadata.caller) {
        caller_event.insert(metadata.caller.clone(), vec![position]);
        return Ok(());
    }

    canister_event
        .get_mut(&metadata.canister)
        .unwrap()
        .push(position);
    caller_event
        .get_mut(&metadata.caller)
        .unwrap()
        .push(position);

    let len = canister_event.get(&metadata.canister).unwrap().len();
    let info = format!("count {}", len);
    print(info);
    Ok(())
}

#[query(name = "getCanisterEvent")]
async fn get_canister_event(canister: Principal, offset: Nat, limit: Nat) -> Vec<metadata::Metadata> {
    let offset = util::nat_to_u64(offset).unwrap() as usize;
    let mut limit = util::nat_to_u64(limit).unwrap() as usize;
    if limit > 50 {
        limit = 50;
    }
    let canister_event = storage::get::<CanisterEvent>();
    if !canister_event.contains_key(&canister) {
        print("canister not exist");
        return vec![];
    }
    let blucket = canister_event.get(&canister).unwrap();
    let info = format!("count {},{},{}", blucket.len(), offset, limit);
    if offset > blucket.len() {
        return vec![];
    }
    let mut container: Vec<metadata::Metadata> = vec![];
    unsafe {
        if offset + limit > blucket.len() {
            for i in blucket[offset..info.len()].iter() {
                container.push(Event[*i].clone());
            }
            return container;
        }

        for i in blucket[offset..offset + limit].iter() {
            container.push(Event[*i].clone());
        }
        return container;
    }
}

/// Get caller transaction transaction record information
#[query(name = "getCallerEvent")]
async fn get_caller_event(caller: Principal, offset: Nat, limit: Nat) -> Vec<metadata::Metadata> {
    let offset = util::nat_to_u64(offset).unwrap() as usize;
    let mut limit = util::nat_to_u64(limit).unwrap() as usize;
    if limit > 50 {
        limit = 50;
    }
    let caller_event = storage::get::<CallerEvent>();
    if !caller_event.contains_key(&caller) {
        print("canister not exist");
        return vec![];
    }
    let blucket = caller_event.get(&caller).unwrap();
    let info = format!("count {},{},{}", blucket.len(), offset, limit);
    if offset > blucket.len() {
        return vec![];
    }
    let mut container: Vec<metadata::Metadata> = vec![];
    unsafe {
        if offset + limit > blucket.len() {
            for i in blucket[offset..info.len()].iter() {
                container.push(Event[*i].clone());
            }
            return container;
        }

        for i in blucket[offset..offset + limit].iter() {
            container.push(Event[*i].clone());
        }
        return container;
    }
}

#[query(name = "getLastEvent")]
async fn get_last_event(limit: Nat) -> Vec<metadata::Metadata> {
    let limit = util::nat_to_u64(limit).unwrap() as usize;
    unsafe {
        let len = Event.len();
        if len <= limit {
            return Event.to_vec();
        }
        return Event[len - limit - 1..len - 1].to_vec();
    }
}

#[query(name = "getCanisterLastEvent")]
async fn get_canister_last_event(canister: Principal, limit: Nat) -> Vec<metadata::Metadata> {
    let limit = util::nat_to_u64(limit).unwrap() as usize;
    let canister_event = storage::get::<CanisterEvent>();
    let mut result: Vec<metadata::Metadata> = vec![];
    if !canister_event.contains_key(&canister) {
        return result;
    }
    let event = canister_event.get(&canister).unwrap();
    let len = event.len();
    unsafe {
        if len <= limit {
            for i in event.iter() {
                result.push(Event[*i].clone())
            }
            return result;
        }
        for j in event[len - limit - 1..len - 1].iter() {
            result.push(Event[*j].clone())
        }
        return result;
    }
}

#[query(name = "getCanisterList")]
async fn get_canister_list() -> Vec<Principal> {
    let mut res: Vec<Principal> = vec![];
    let canister_event = storage::get::<CanisterEvent>();
    for keys in canister_event.keys() {
        res.push(keys.clone());
    }
    return res;
}

/// Before the upgrade task starts, you need to persist the data in memory
#[pre_upgrade]
fn pre_upgrade() {
    let mut canister_event_snapshot: Vec<Snapshot<Vec<usize>>> = vec![];
    let mut caller_event_snapshot: Vec<Snapshot<Vec<usize>>> = vec![];
    let canister_event = storage::get::<CanisterEvent>();
    for (k, v) in canister_event.iter() {
        let snapshot = Snapshot::<Vec<usize>> {
            key: k.clone(),
            value: v.to_vec(),
        };
        canister_event_snapshot.push(snapshot);
    }
    let caller_event = storage::get::<CallerEvent>();

    for (k, v) in caller_event.iter() {
        let snapshot = Snapshot::<Vec<usize>> {
            key: k.clone(),
            value: v.to_vec(),
        };
        caller_event_snapshot.push(snapshot);
    }

    unsafe {
        let db = Db {
            canisterEvent: canister_event_snapshot,
            callerEvent: caller_event_snapshot,
            event: Event.to_vec(),
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
        Event = db.event;
    }
    let canister_event = storage::get_mut::<CanisterEvent>();
    for e in db.canisterEvent.into_iter() {
        canister_event.insert(e.key.clone(), e.value);
    }
    for e in db.callerEvent.into_iter() {
        canister_event.insert(e.key.clone(), e.value);
    }
}
