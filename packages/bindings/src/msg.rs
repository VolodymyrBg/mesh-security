use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, CosmosMsg, CustomMsg, Uint128};

/// A top-level Custom message for the token factory.
/// It is embedded like this to easily allow adding other variants that are custom
/// to your chain, or other "standardized" extensions along side it.
#[cw_serde]
pub enum VirtualStakeCustomMsg {
    VirtualStake(VirtualStakeMsg),
}

/// Special messages to be supported by any chain that supports token_factory
#[cw_serde]
pub enum VirtualStakeMsg {
    /// Bond will enforce the calling contract has a max cap established.
    /// It ensures amount.denom is the native staking denom,
    /// and that (currently minted + amount.amount <= max_cap)
    ///
    /// If these conditions are met, it will mint amount.amount tokens
    /// to the caller's account and delegate them to the named validator.
    /// It will also update the currently minted amount.
    Bond { amount: Coin, validator: String },
    /// Unbond ensures that amount.denom is the native staking denom,
    /// that caller is able to mint and (currently minted >= amount.amount).
    /// It also checks that the caller has at least amoubt.amount tokens
    /// currently bonded to the named validator.
    ///
    /// If these conditions are met, it will instantly undelegate
    /// amount.amount tokens from the caller to the named validator.
    /// It will then burn those tokens from the caller's account,
    /// and update the currently minted amount.
    Unbond { amount: Coin, validator: String },
    /// TODO: Add docs for this msg
    /// 
    UpdateDelegation { amount: Coin, is_deduct: bool, delegator: String, validator: String},
}

impl VirtualStakeMsg {
    pub fn bond(denom: &str, amount: impl Into<Uint128>, validator: &str) -> VirtualStakeMsg {
        let coin = Coin {
            amount: amount.into(),
            denom: denom.into(),
        };
        VirtualStakeMsg::Bond {
            amount: coin,
            validator: validator.to_string(),
        }
    }

    pub fn unbond(denom: &str, amount: impl Into<Uint128>, validator: &str) -> VirtualStakeMsg {
        let coin = Coin {
            amount: amount.into(),
            denom: denom.into(),
        };
        VirtualStakeMsg::Unbond {
            amount: coin,
            validator: validator.to_string(),
        }
    }
    
    pub fn update_delegation(denom: &str, is_deduct: bool, amount: impl Into<Uint128>, delgator: &str, validator: &str) -> VirtualStakeMsg {
        let coin = Coin {
            amount: amount.into(),
            denom: denom.into(),
        };
        VirtualStakeMsg::UpdateDelegation {
            amount: coin,
            is_deduct,
            delegator: delgator.to_string(),
            validator: validator.to_string(),
        }
    }
}

impl From<VirtualStakeMsg> for CosmosMsg<VirtualStakeCustomMsg> {
    fn from(msg: VirtualStakeMsg) -> CosmosMsg<VirtualStakeCustomMsg> {
        CosmosMsg::Custom(VirtualStakeCustomMsg::VirtualStake(msg))
    }
}

impl CustomMsg for VirtualStakeCustomMsg {}
