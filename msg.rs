use crate::state::UnparsedRecipient;
use cosmwasm_std::{Coin, Uint128};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ResetTimestamp {},
    DepositTokens {},
    WithdrawTokens { tokens: Vec<Coin> },
    DistributeAssets { owner: String },
    SetRecipients { recipients: Vec<UnparsedRecipient> },
    SetRenewalRate { renewal_rate: Uint128 }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    SeeMemberships { address: String },
    GetWill { address: String },
}
