#![cfg(test)]

use soroban_sdk::{
    testutils::{
        Address as _,
        BytesN as _,
    },
    token::StellarAssetClient,
    Address,
    BytesN,
    Env,
    String,
};

use blend_contract_sdk::{
    pool,
    testutils::{
        default_reserve_config,
        BlendFixture,
    },
};

use crate::contract::{
    RemiVaultCore,
    RemiVaultCoreClient,
};

// =====================================================
// PHASE 1 TEST
// Blend fixture + pool setup
// =====================================================

#[test]
fn test_blend_fixture_pool_setup() {
    let env = Env::default();

    let deployer =
        Address::generate(&env);

    let blnd =
        env.register_stellar_asset_contract_v2(
            deployer.clone(),
        )
        .address();

    let usdc =
        env.register_stellar_asset_contract_v2(
            deployer.clone(),
        )
        .address();

    let blend =
        BlendFixture::deploy(
            &env,
            &deployer,
            &blnd,
            &usdc,
        );

    let pool_address =
        blend.pool_factory.mock_all_auths().deploy(
            &deployer,
            &String::from_str(&env, "remivault-test"),
            &BytesN::<32>::random(&env),
            &Address::generate(&env),
            &0_1000000,
            &4,
            &1_0000000,
        );

    let pool_client =
        pool::Client::new(
            &env,
            &pool_address,
        );

    let reserve_config =
        default_reserve_config();

    pool_client
        .mock_all_auths()
        .queue_set_reserve(
            &usdc,
            &reserve_config,
        );

    pool_client
        .mock_all_auths()
        .set_reserve(&usdc);

    blend
        .backstop
        .mock_all_auths()
        .deposit(
            &deployer,
            &pool_address,
            &50_000_0000000,
        );

    pool_client
        .mock_all_auths()
        .set_status(&3);

    let status =
        pool_client
            .mock_all_auths()
            .update_status();

    assert_eq!(status, 1);

    assert!(
        blend.pool_factory.is_pool(
            &pool_address,
        )
    );

    let reserve =
        pool_client.get_reserve(&usdc);

    assert_eq!(
        reserve.config.enabled,
        true,
    );
}

// =====================================================
// PHASE 2 TEST
// RemiVault deposit -> Blend collateral increases
// =====================================================

#[test]
fn test_deposit_supplies_to_blend_and_mints_shares() {
    let env = Env::default();
    env.mock_all_auths();

    let deployer = Address::generate(&env);
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let user = Address::generate(&env);

    let blnd = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let usdc = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let usdc_admin =
        StellarAssetClient::new(&env, &usdc);

    usdc_admin.mint(
        &user,
        &1_000_0000000,
    );

    let blend =
        BlendFixture::deploy(
            &env,
            &deployer,
            &blnd,
            &usdc,
        );

    let pool_address =
        blend.pool_factory.mock_all_auths().deploy(
            &deployer,
            &String::from_str(&env, "remivault-test"),
            &BytesN::<32>::random(&env),
            &Address::generate(&env),
            &0_1000000,
            &4,
            &1_0000000,
        );

    let pool_client =
        pool::Client::new(&env, &pool_address);

    let reserve_config =
        default_reserve_config();

    pool_client
        .mock_all_auths()
        .queue_set_reserve(&usdc, &reserve_config);

    pool_client
        .mock_all_auths()
        .set_reserve(&usdc);

    blend
        .backstop
        .mock_all_auths()
        .deposit(
            &deployer,
            &pool_address,
            &50_000_0000000,
        );

    pool_client
        .mock_all_auths()
        .set_status(&3);

    assert_eq!(
        pool_client.mock_all_auths().update_status(),
        1,
    );

    let vault_address =
        env.register(
            RemiVaultCore,
            (),
        );

    let vault_client =
        RemiVaultCoreClient::new(
            &env,
            &vault_address,
        );

    vault_client.initialize(
        &usdc,
        &admin,
        &treasury,
        &pool_address,
        &1500,
    );

    let reserve =
        pool_client.get_reserve(&usdc);

    let reserve_index =
        reserve.config.index;

    let before_positions =
        pool_client.get_positions(&vault_address);

    let before_collateral =
        before_positions
            .collateral
            .get(reserve_index)
            .unwrap_or(0);

    assert_eq!(before_collateral, 0);

    let deposit_amount =
        100_0000000;

    vault_client.deposit(
        &user,
        &deposit_amount,
    );

    let after_positions =
        pool_client.get_positions(&vault_address);

    let after_collateral =
        after_positions
            .collateral
            .get(reserve_index)
            .unwrap_or(0);

    assert!(
        after_collateral > 0,
        "Vault collateral should increase in Blend after deposit"
    );

    let user_shares =
        vault_client.get_user_balance(&user);

    assert!(
        user_shares > 0,
        "User should receive vault shares after deposit"
    );
}

#[test]
fn test_withdraw_reduces_user_shares_and_blend_collateral() {
    let env = Env::default();
    env.mock_all_auths();

    let deployer = Address::generate(&env);
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let user = Address::generate(&env);

    let blnd = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let usdc = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let usdc_admin =
        StellarAssetClient::new(&env, &usdc);

    usdc_admin.mint(
        &user,
        &1_000_0000000,
    );

    let token_client =
        soroban_sdk::token::Client::new(
            &env,
            &usdc,
        );

    let blend =
        BlendFixture::deploy(
            &env,
            &deployer,
            &blnd,
            &usdc,
        );

    let pool_address =
        blend.pool_factory.mock_all_auths().deploy(
            &deployer,
            &String::from_str(&env, "remivault-test"),
            &BytesN::<32>::random(&env),
            &Address::generate(&env),
            &0_1000000,
            &4,
            &1_0000000,
        );

    let pool_client =
        pool::Client::new(&env, &pool_address);

    let reserve_config =
        default_reserve_config();

    pool_client
        .mock_all_auths()
        .queue_set_reserve(&usdc, &reserve_config);

    pool_client
        .mock_all_auths()
        .set_reserve(&usdc);

    blend
        .backstop
        .mock_all_auths()
        .deposit(
            &deployer,
            &pool_address,
            &50_000_0000000,
        );

    pool_client
        .mock_all_auths()
        .set_status(&3);

    assert_eq!(
        pool_client.mock_all_auths().update_status(),
        1,
    );

    let vault_address =
        env.register(
            RemiVaultCore,
            (),
        );

    let vault_client =
        RemiVaultCoreClient::new(
            &env,
            &vault_address,
        );

    vault_client.initialize(
        &usdc,
        &admin,
        &treasury,
        &pool_address,
        &1500,
    );

    let deposit_amount =
        100_0000000;

    vault_client.deposit(
        &user,
        &deposit_amount,
    );

    let shares_before =
        vault_client.get_user_balance(&user);

    assert!(shares_before > 0);

    let reserve =
        pool_client.get_reserve(&usdc);

    let reserve_index =
        reserve.config.index;

    let positions_before =
        pool_client.get_positions(&vault_address);

    let collateral_before =
        positions_before
            .collateral
            .get(reserve_index)
            .unwrap_or(0);

    let user_usdc_before_withdraw =
        token_client.balance(&user);

    let shares_to_withdraw =
        shares_before / 2;

    vault_client.withdraw(
        &user,
        &shares_to_withdraw,
    );

    let shares_after =
        vault_client.get_user_balance(&user);

    assert!(
        shares_after < shares_before,
        "User shares should reduce after withdraw"
    );

    let positions_after =
        pool_client.get_positions(&vault_address);

    let collateral_after =
        positions_after
            .collateral
            .get(reserve_index)
            .unwrap_or(0);

    assert!(
        collateral_after < collateral_before,
        "Blend collateral should reduce after withdraw"
    );

    let user_usdc_after_withdraw =
        token_client.balance(&user);

    assert!(
        user_usdc_after_withdraw > user_usdc_before_withdraw,
        "User USDC balance should increase after withdraw"
    );
}