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

use remivault_core::contract::{
    RemiVaultCore,
    RemiVaultCoreClient,
};

use crate::contract::{
    RemiVaultAdmin,
    RemiVaultAdminClient,
};

// =====================================================
// ADMIN HARVEST TEST
// Checks admin can read Blend position and call harvest()
// =====================================================

#[test]
fn test_admin_harvest_after_vault_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let deployer =
        Address::generate(&env);

    let admin =
        Address::generate(&env);

    let treasury =
        Address::generate(&env);

    let user =
        Address::generate(&env);

    // -------------------------------------------------
    // Deploy test assets
    // -------------------------------------------------

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

    let usdc_admin =
        StellarAssetClient::new(
            &env,
            &usdc,
        );

    usdc_admin.mint(
        &user,
        &1_000_0000000,
    );

    // -------------------------------------------------
    // Deploy Blend fixture and pool
    // -------------------------------------------------

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
            &String::from_str(&env, "remivault-admin-test"),
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

    assert_eq!(
        pool_client
            .mock_all_auths()
            .update_status(),
        1,
    );

    // -------------------------------------------------
    // Deploy Core Vault
    // -------------------------------------------------

    let core_vault_address =
        env.register(
            RemiVaultCore,
            (),
        );

    let core_client =
        RemiVaultCoreClient::new(
            &env,
            &core_vault_address,
        );

    core_client.initialize(
        &usdc,
        &admin,
        &treasury,
        &pool_address,
        &1500,
    );

    // -------------------------------------------------
    // User deposits into Core Vault
    // -------------------------------------------------

    core_client.deposit(
        &user,
        &100_0000000,
    );

    let user_shares =
        core_client.get_user_balance(
            &user,
        );

    assert!(
        user_shares > 0,
        "User should have shares before admin harvest"
    );

    // -------------------------------------------------
    // Confirm vault has Blend collateral
    // -------------------------------------------------

    let reserve =
        pool_client.get_reserve(&usdc);

    let reserve_index =
        reserve.config.index;

    let positions =
        pool_client.get_positions(
            &core_vault_address,
        );

    let collateral =
        positions
            .collateral
            .get(reserve_index)
            .unwrap_or(0);

    assert!(
        collateral > 0,
        "Core vault should have collateral in Blend"
    );

    // -------------------------------------------------
    // Deploy Admin Contract
    // -------------------------------------------------

    let admin_contract_address =
        env.register(
            RemiVaultAdmin,
            (),
        );

    let admin_client =
        RemiVaultAdminClient::new(
            &env,
            &admin_contract_address,
        );

    admin_client.initialize(
        &admin,
        &core_vault_address,
        &usdc,
        &treasury,
        &pool_address,
        &1500,
    );

    // -------------------------------------------------
    // Call admin harvest
    // -------------------------------------------------

    admin_client.harvest();

    // -------------------------------------------------
    // Check core vault state still exists
    // -------------------------------------------------

    let state =
        core_client.vault_state();

    assert!(
        state.total_assets > 0,
        "Vault total assets should remain positive after harvest"
    );
}
#[test]
fn test_collect_fees_transfers_to_treasury() {
    let env = Env::default();
    env.mock_all_auths();

    let deployer = Address::generate(&env);
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);

    let blnd = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let usdc = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let usdc_admin =
        StellarAssetClient::new(&env, &usdc);

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
            &String::from_str(&env, "fee-test"),
            &BytesN::<32>::random(&env),
            &Address::generate(&env),
            &0_1000000,
            &4,
            &1_0000000,
        );

    let core_vault_address =
        env.register(
            RemiVaultCore,
            (),
        );

    let core_client =
        RemiVaultCoreClient::new(
            &env,
            &core_vault_address,
        );

    core_client.initialize(
        &usdc,
        &admin,
        &treasury,
        &pool_address,
        &1500,
    );

    let admin_contract_address =
        env.register(
            RemiVaultAdmin,
            (),
        );

    let admin_client =
        RemiVaultAdminClient::new(
            &env,
            &admin_contract_address,
        );

    admin_client.initialize(
        &admin,
        &core_vault_address,
        &usdc,
        &treasury,
        &pool_address,
        &1500,
    );

    // -------------------------------------------------
    // Seed fees inside core vault
    // -------------------------------------------------

    let platform_fee =
        10_0000000;

    // Give core vault liquid USDC so fee transfer can happen
    usdc_admin.mint(
        &core_vault_address,
        &platform_fee,
    );

    // Add accrued fee into core vault accounting
    core_client._harvest_internal(
        &0,
        &platform_fee,
    );

    let token_client =
        soroban_sdk::token::Client::new(
            &env,
            &usdc,
        );

    let treasury_before =
        token_client.balance(&treasury);

    admin_client.collect_fees();

    let treasury_after =
        token_client.balance(&treasury);

    assert!(
        treasury_after > treasury_before,
        "Treasury should receive collected fees"
    );

    assert_eq!(
        treasury_after - treasury_before,
        platform_fee,
        "Treasury should receive exact platform fee amount"
    );
}
#[test]
fn test_admin_pause_and_unpause_vault() {
    let env = Env::default();
    env.mock_all_auths();

    let deployer = Address::generate(&env);
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);

    let blnd = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let usdc = env
        .register_stellar_asset_contract_v2(deployer.clone())
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
            &String::from_str(&env, "pause-test"),
            &BytesN::<32>::random(&env),
            &Address::generate(&env),
            &0_1000000,
            &4,
            &1_0000000,
        );

    let core_vault_address =
        env.register(
            RemiVaultCore,
            (),
        );

    let core_client =
        RemiVaultCoreClient::new(
            &env,
            &core_vault_address,
        );

    core_client.initialize(
        &usdc,
        &admin,
        &treasury,
        &pool_address,
        &1500,
    );

    let admin_contract_address =
        env.register(
            RemiVaultAdmin,
            (),
        );

    let admin_client =
        RemiVaultAdminClient::new(
            &env,
            &admin_contract_address,
        );

    admin_client.initialize(
        &admin,
        &core_vault_address,
        &usdc,
        &treasury,
        &pool_address,
        &1500,
    );

    admin_client.pause(&2);

    let state_after_pause =
        core_client.vault_state();

    assert!(
        state_after_pause.total_assets >= 0,
        "Vault state should still be readable after pause"
    );

    admin_client.unpause();

    let state_after_unpause =
        core_client.vault_state();

    assert!(
        state_after_unpause.total_assets >= 0,
        "Vault state should still be readable after unpause"
    );
}
#[test]
fn test_admin_update_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let deployer = Address::generate(&env);
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);

    let usdc = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let blnd = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let blend =
        BlendFixture::deploy(&env, &deployer, &blnd, &usdc);

    let pool_address =
        blend.pool_factory.mock_all_auths().deploy(
            &deployer,
            &String::from_str(&env, "fee-update-test"),
            &BytesN::<32>::random(&env),
            &Address::generate(&env),
            &0_1000000,
            &4,
            &1_0000000,
        );

    let core_vault_address =
        env.register(RemiVaultCore, ());

    let core_client =
        RemiVaultCoreClient::new(&env, &core_vault_address);

    core_client.initialize(
        &usdc,
        &admin,
        &treasury,
        &pool_address,
        &1500,
    );

    let admin_contract_address =
        env.register(RemiVaultAdmin, ());

    let admin_client =
        RemiVaultAdminClient::new(&env, &admin_contract_address);

    admin_client.initialize(
        &admin,
        &core_vault_address,
        &usdc,
        &treasury,
        &pool_address,
        &1500,
    );

    admin_client.update_fee(&2000);

    // If this does not panic, admin → core fee update bridge works.
    let state =
        core_client.vault_state();

    assert!(
        state.total_assets >= 0,
        "Vault should remain readable after fee update"
    );
}
#[test]
fn test_admin_update_treasury() {
    let env = Env::default();
    env.mock_all_auths();

    let deployer = Address::generate(&env);
    let admin = Address::generate(&env);
    let old_treasury = Address::generate(&env);
    let new_treasury = Address::generate(&env);

    let usdc = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let blnd = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let blend =
        BlendFixture::deploy(&env, &deployer, &blnd, &usdc);

    let pool_address =
        blend.pool_factory.mock_all_auths().deploy(
            &deployer,
            &String::from_str(&env, "treasury-update-test"),
            &BytesN::<32>::random(&env),
            &Address::generate(&env),
            &0_1000000,
            &4,
            &1_0000000,
        );

    let core_vault_address =
        env.register(RemiVaultCore, ());

    let core_client =
        RemiVaultCoreClient::new(&env, &core_vault_address);

    core_client.initialize(
        &usdc,
        &admin,
        &old_treasury,
        &pool_address,
        &1500,
    );

    let admin_contract_address =
        env.register(RemiVaultAdmin, ());

    let admin_client =
        RemiVaultAdminClient::new(&env, &admin_contract_address);

    admin_client.initialize(
        &admin,
        &core_vault_address,
        &usdc,
        &old_treasury,
        &pool_address,
        &1500,
    );

    admin_client.update_treasury(&new_treasury);

    let state =
        core_client.vault_state();

    assert!(
        state.total_assets >= 0,
        "Vault should remain readable after treasury update"
    );
}
#[test]
fn test_admin_migrate_blend_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let deployer = Address::generate(&env);
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);

    let usdc = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let blnd = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let blend =
        BlendFixture::deploy(&env, &deployer, &blnd, &usdc);

    let old_pool =
        blend.pool_factory.mock_all_auths().deploy(
            &deployer,
            &String::from_str(&env, "old-pool"),
            &BytesN::<32>::random(&env),
            &Address::generate(&env),
            &0_1000000,
            &4,
            &1_0000000,
        );

    let new_pool =
        blend.pool_factory.mock_all_auths().deploy(
            &deployer,
            &String::from_str(&env, "new-pool"),
            &BytesN::<32>::random(&env),
            &Address::generate(&env),
            &0_1000000,
            &4,
            &1_0000000,
        );

    let core_vault_address =
        env.register(RemiVaultCore, ());

    let core_client =
        RemiVaultCoreClient::new(&env, &core_vault_address);

    core_client.initialize(
        &usdc,
        &admin,
        &treasury,
        &old_pool,
        &1500,
    );

    let admin_contract_address =
        env.register(RemiVaultAdmin, ());

    let admin_client =
        RemiVaultAdminClient::new(&env, &admin_contract_address);

    admin_client.initialize(
        &admin,
        &core_vault_address,
        &usdc,
        &treasury,
        &old_pool,
        &1500,
    );

    admin_client.migrate_blend_pool(&new_pool);

    let state =
        core_client.vault_state();

    assert!(
        state.total_assets >= 0,
        "Vault should remain readable after Blend pool migration"
    );
}
#[test]
#[should_panic]
fn test_admin_rejects_fee_above_5000_bps() {
    let env = Env::default();
    env.mock_all_auths();

    let deployer = Address::generate(&env);
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);

    let usdc = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let blnd = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let blend =
        BlendFixture::deploy(&env, &deployer, &blnd, &usdc);

    let pool_address =
        blend.pool_factory.mock_all_auths().deploy(
            &deployer,
            &String::from_str(&env, "invalid-fee-test"),
            &BytesN::<32>::random(&env),
            &Address::generate(&env),
            &0_1000000,
            &4,
            &1_0000000,
        );

    let core_vault_address =
        env.register(RemiVaultCore, ());

    let core_client =
        RemiVaultCoreClient::new(&env, &core_vault_address);

    core_client.initialize(
        &usdc,
        &admin,
        &treasury,
        &pool_address,
        &1500,
    );

    let admin_contract_address =
        env.register(RemiVaultAdmin, ());

    let admin_client =
        RemiVaultAdminClient::new(&env, &admin_contract_address);

    admin_client.initialize(
        &admin,
        &core_vault_address,
        &usdc,
        &treasury,
        &pool_address,
        &1500,
    );

    admin_client.update_fee(&5001);
}
#[test]
#[should_panic]
fn test_pause_blocks_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let deployer = Address::generate(&env);
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let user = Address::generate(&env);

    let usdc = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let blnd = env
        .register_stellar_asset_contract_v2(deployer.clone())
        .address();

    let usdc_admin =
        StellarAssetClient::new(&env, &usdc);

    usdc_admin.mint(&user, &1_000_0000000);

    let blend =
        BlendFixture::deploy(&env, &deployer, &blnd, &usdc);

    let pool_address =
        blend.pool_factory.mock_all_auths().deploy(
            &deployer,
            &String::from_str(&env, "pause-block-test"),
            &BytesN::<32>::random(&env),
            &Address::generate(&env),
            &0_1000000,
            &4,
            &1_0000000,
        );

    let core_vault_address =
        env.register(RemiVaultCore, ());

    let core_client =
        RemiVaultCoreClient::new(&env, &core_vault_address);

    core_client.initialize(
        &usdc,
        &admin,
        &treasury,
        &pool_address,
        &1500,
    );

    let admin_contract_address =
        env.register(RemiVaultAdmin, ());

    let admin_client =
        RemiVaultAdminClient::new(&env, &admin_contract_address);

    admin_client.initialize(
        &admin,
        &core_vault_address,
        &usdc,
        &treasury,
        &pool_address,
        &1500,
    );

    admin_client.pause(&2);

    core_client.deposit(
        &user,
        &100_0000000,
    );
}