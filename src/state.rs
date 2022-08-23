use schemars::JsonSchema;
use cosmwasm_std::{Addr, Decimal};
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub fee: Decimal,
    pub owner: Addr,
    pub fee_receiver: Addr,
}

pub const STATE: Item<State> = Item::new("state");
