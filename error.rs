use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InsufficientFunds")]
    InsufficientFunds {},

    #[error("NonExistentWill")]
    NonExistentWill {},

    #[error("InvalidRecipients")]
    InvalidRecipients {},

    #[error("NotClaimable")]
    NotClaimable {},

    #[error("InvalidTokenDenom")]
    InvalidTokenDenom {},
}
