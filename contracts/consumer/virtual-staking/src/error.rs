use cosmwasm_std::{StdError, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Wrong denom. Cannot stake {0}")]
    WrongDenom(String),

    #[error("Cannot unbond {1} tokens from validator {0}, not enough staked")]
    InsufficientBond(String, Uint128),

    #[error("Invalid Reply ID. Don't recognize {0}")]
    InvalidReplyId(u64),

    #[error("Empty validators list")]
    NoValidators {},

    #[error("Virtual staking {0} has not enough delegated funds: {1}")]
    InsufficientDelegations(String, Uint128),
}
