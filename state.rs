use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::{Item, Map};

//TODO: CANONICAL_ADDR
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub admin: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Will {
    pub owner: Addr,
    pub recipients: Vec<Recipient>,
    pub timestamp: Uint128,
    pub renewal_rate: Uint128,
    pub tokens: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Recipient {
    pub address: Addr,
    pub percentage: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnparsedRecipient {
    pub address: String,
    pub percentage: Uint128,
}

pub const STATE: Item<State> = Item::new("state");
pub const WILLS: Map<Addr, Will> = Map::new("wills"); // Might want to use IndexedMap
pub const MEMBERSHIPS: Map<Addr, Vec<Addr>> = Map::new("withinwills");
