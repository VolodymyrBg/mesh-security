use cosmwasm_std::{coin, coins, to_binary, Addr, Decimal, StdError, Uint128};

use cw_multi_test::App as MtApp;
use sylvia::multitest::App;

use mesh_sync::ValueRange;

use crate::local_staking_api::test_utils::LocalStakingApi;
use crate::native_staking_callback::test_utils::NativeStakingCallback;

mod local_staking_proxy;

use crate::contract;
use crate::error::ContractError;
use crate::msg;
use crate::msg::{OwnerByProxyResponse, ProxyByOwnerResponse};

const OSMO: &str = "OSMO";

const SLASHING_PERCENTAGE: u64 = 15;

fn slashing_rate() -> Decimal {
    Decimal::percent(SLASHING_PERCENTAGE)
}

#[test]
fn instantiation() {
    let app = App::default();

    let owner = "vault"; // Owner of the staking contract (i. e. the vault contract)

    let staking_proxy_code = local_staking_proxy::multitest_utils::CodeId::store_code(&app);
    let staking_code = contract::multitest_utils::CodeId::store_code(&app);

    let staking = staking_code
        .instantiate(
            OSMO.to_owned(),
            staking_proxy_code.code_id(),
            slashing_rate(),
        )
        .with_label("Staking")
        .call(owner)
        .unwrap();

    let config = staking.config().unwrap();
    assert_eq!(config.denom, OSMO);

    let res = staking.local_staking_api_proxy().max_slash().unwrap();
    assert_eq!(res.max_slash, slashing_rate());
}

#[test]
fn receiving_stake() {
    let owner = "vault"; // Owner of the staking contract (i. e. the vault contract)

    let user1 = "user1"; // One who wants to local stake
    let user2 = "user2"; // Another one who wants to local stake

    let validator = "validator1"; // Validator to stake on

    // Fund the vault
    let app = MtApp::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &Addr::unchecked(owner), coins(300, OSMO))
            .unwrap();
    });
    let app = App::new(app);

    // Contracts setup
    let staking_proxy_code = local_staking_proxy::multitest_utils::CodeId::store_code(&app);
    let staking_code = contract::multitest_utils::CodeId::store_code(&app);

    let staking = staking_code
        .instantiate(
            OSMO.to_owned(),
            staking_proxy_code.code_id(),
            slashing_rate(),
        )
        .with_label("Staking")
        .call(owner)
        .unwrap();

    // Check that no proxy exists for user1 yet
    let err = staking.proxy_by_owner(user1.to_owned()).unwrap_err();
    assert!(matches!(
        err,
        ContractError::Std(StdError::GenericErr { .. }) // Addr not found
    ));

    // Receive some stake on behalf of user1 for validator
    let stake_msg = to_binary(&msg::StakeMsg {
        validator: validator.to_owned(),
    })
    .unwrap();
    staking
        .local_staking_api_proxy()
        .receive_stake(user1.to_owned(), stake_msg)
        .with_funds(&coins(100, OSMO))
        .call(owner) // called from vault
        .unwrap();

    let proxy1 = staking.proxy_by_owner(user1.to_owned()).unwrap().proxy;
    // Reverse query
    assert_eq!(
        staking.owner_by_proxy(proxy1.clone()).unwrap(),
        OwnerByProxyResponse {
            owner: user1.to_owned(),
        }
    );

    // Check that funds are in the proxy contract
    assert_eq!(
        app.app()
            .wrap()
            .query_balance(proxy1.clone(), OSMO)
            .unwrap(),
        coin(100, OSMO)
    );

    // Stake some more
    let stake_msg = to_binary(&msg::StakeMsg {
        validator: validator.to_owned(),
    })
    .unwrap();
    staking
        .local_staking_api_proxy()
        .receive_stake(user1.to_owned(), stake_msg)
        .with_funds(&coins(50, OSMO))
        .call(owner) // called from vault
        .unwrap();

    // Check that same proxy is used
    assert_eq!(
        staking.proxy_by_owner(user1.to_owned()).unwrap(),
        ProxyByOwnerResponse {
            proxy: proxy1.clone(),
        }
    );

    // Reverse check
    assert_eq!(
        staking.owner_by_proxy(proxy1.clone()).unwrap(),
        OwnerByProxyResponse {
            owner: user1.to_owned(),
        }
    );

    // Check that funds are updated in the proxy contract
    assert_eq!(
        app.app().wrap().query_balance(proxy1, OSMO).unwrap(),
        coin(150, OSMO)
    );

    // Receive some stake on behalf of user2 for validator
    let stake_msg = to_binary(&msg::StakeMsg {
        validator: validator.to_owned(),
    })
    .unwrap();
    staking
        .local_staking_api_proxy()
        .receive_stake(user2.to_owned(), stake_msg)
        .with_funds(&coins(10, OSMO))
        .call(owner) // called from vault
        .unwrap();

    let proxy2 = staking.proxy_by_owner(user2.to_owned()).unwrap().proxy;
    // Reverse query
    assert_eq!(
        staking.owner_by_proxy(proxy2.to_string()).unwrap(),
        OwnerByProxyResponse {
            owner: user2.to_owned(),
        }
    );

    // Check that funds are in the corresponding proxy contract
    assert_eq!(
        app.app().wrap().query_balance(proxy2, OSMO).unwrap(),
        coin(10, OSMO)
    );
}

#[test]
fn releasing_proxy_stake() {
    let owner = "vault_admin"; // Owner of the vault contract

    let vault_addr = "contract0"; // First created contract
    let staking_addr = "contract1"; // Second contract (instantiated by vault)
    let proxy_addr = "contract2"; // Staking proxy contract for user1 (instantiated by staking contract on stake)

    let user = "user1"; // One who wants to release stake
    let validator = "validator1";

    // Fund the user
    let app = MtApp::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &Addr::unchecked(user), coins(300, OSMO))
            .unwrap();
    });
    let app = App::new(app);

    // Contracts setup
    let vault_code = mesh_vault::contract::multitest_utils::CodeId::store_code(&app);
    let staking_code = contract::multitest_utils::CodeId::store_code(&app);
    let staking_proxy_code = local_staking_proxy::multitest_utils::CodeId::store_code(&app);

    // Instantiate vault msg
    let staking_init_info = mesh_vault::msg::StakingInitInfo {
        admin: None,
        code_id: staking_code.code_id(),
        msg: to_binary(&crate::contract::InstantiateMsg {
            denom: OSMO.to_owned(),
            proxy_code_id: staking_proxy_code.code_id(),
            max_slashing: slashing_rate(),
        })
        .unwrap(),
        label: None,
    };

    // Instantiates vault and staking contracts
    let vault = vault_code
        .instantiate(OSMO.to_owned(), Some(staking_init_info))
        .with_label("Vault")
        .call(owner)
        .unwrap();

    // Vault is empty
    assert_eq!(
        app.app().wrap().query_balance(vault_addr, OSMO).unwrap(),
        coin(0, OSMO)
    );

    // Access staking instance
    let staking = contract::multitest_utils::NativeStakingContractProxy::new(
        Addr::unchecked(staking_addr),
        &app,
    );

    // User bonds some funds to the vault
    vault
        .bond()
        .with_funds(&coins(200, OSMO))
        .call(user)
        .unwrap();

    // Vault has the funds
    assert_eq!(
        app.app().wrap().query_balance(vault_addr, OSMO).unwrap(),
        coin(200, OSMO)
    );

    // Stakes some of it locally, to validator. This instantiates the staking proxy contract for
    // user
    vault
        .stake_local(
            coin(100, OSMO),
            to_binary(&msg::StakeMsg {
                validator: validator.to_owned(),
            })
            .unwrap(),
        )
        .call(user)
        .unwrap();

    // Vault has half the funds
    assert_eq!(
        app.app().wrap().query_balance(vault_addr, OSMO).unwrap(),
        coin(100, OSMO)
    );

    // And a lien on the other half
    let claims = vault.account_claims(user.to_owned(), None, None).unwrap();
    assert_eq!(
        claims.claims,
        [mesh_vault::msg::LienResponse {
            lienholder: staking_addr.to_owned(),
            amount: ValueRange::new_val(Uint128::new(100))
        }]
    );

    // The other half is in the user's proxy contract
    assert_eq!(
        app.app().wrap().query_balance(proxy_addr, OSMO).unwrap(),
        coin(100, OSMO)
    );

    // Now release the funds (as if called from the user's staking proxy)
    staking
        .native_staking_callback_proxy()
        .release_proxy_stake()
        .with_funds(&coins(100, OSMO))
        .call(proxy_addr)
        .unwrap();

    // Check that the vault has the funds again
    assert_eq!(
        app.app().wrap().query_balance(vault_addr, OSMO).unwrap(),
        coin(200, OSMO)
    );
    // And there are no more liens
    let claims = vault.account_claims(user.to_owned(), None, None).unwrap();
    assert_eq!(
        claims.claims,
        [mesh_vault::msg::LienResponse {
            lienholder: staking_addr.to_owned(),
            amount: ValueRange::new_val(Uint128::zero()) // TODO? Clean-up empty liens
        }]
    );
}
