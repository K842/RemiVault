use soroban_sdk::{
    token,
    Address,
    Env,
    Vec,
};

use blend_contract_sdk::pool;

// =====================================================
// BLEND REQUEST TYPE CONSTANTS
// Source: Blend pool actions.rs
// =====================================================

pub const REQUEST_TYPE_SUPPLY_COLLATERAL: u32 = 2;
pub const REQUEST_TYPE_WITHDRAW_COLLATERAL: u32 = 3;

// =====================================================
// APPROVAL LEDGER BUFFER
// =====================================================

const APPROVAL_LEDGER_BUFFER: u32 = 10_000;

// =====================================================
// SUPPLY ASSETS TO BLEND
// =====================================================

pub fn supply_to_blend(
    e: &Env,
    blend_pool: &Address,
    asset: &Address,
    amount: i128,
) -> pool::Positions {
    let vault_address =
        e.current_contract_address();

    let token_client =
        token::Client::new(
            e,
            asset,
        );

    let expiration_ledger =
        e.ledger()
            .sequence()
            + APPROVAL_LEDGER_BUFFER;

    token_client.approve(
        &vault_address,
        blend_pool,
        &amount,
        &expiration_ledger,
    );

    let request =
        pool::Request {
            request_type: REQUEST_TYPE_SUPPLY_COLLATERAL,
            address: asset.clone(),
            amount,
        };

    let mut requests =
        Vec::new(e);

    requests.push_back(request);

    let pool_client =
        pool::Client::new(
            e,
            blend_pool,
        );

    pool_client.submit_with_allowance(
        &vault_address,
        &vault_address,
        &vault_address,
        &requests,
    )
}

// =====================================================
// WITHDRAW ASSETS FROM BLEND
// =====================================================

pub fn withdraw_from_blend(
    e: &Env,
    blend_pool: &Address,
    asset: &Address,
    amount: i128,
) -> pool::Positions {
    let vault_address =
        e.current_contract_address();

    let request =
        pool::Request {
            request_type: REQUEST_TYPE_WITHDRAW_COLLATERAL,
            address: asset.clone(),
            amount,
        };

    let mut requests =
        Vec::new(e);

    requests.push_back(request);

    let pool_client =
        pool::Client::new(
            e,
            blend_pool,
        );

    pool_client.submit(
        &vault_address,
        &vault_address,
        &vault_address,
        &requests,
    )
}

// =====================================================
// GET VAULT BLEND POSITION VALUE
// =====================================================

pub fn get_total_managed_assets(
    e: &Env,
    blend_pool: &Address,
    asset: &Address,
    vault_address: &Address,
) -> i128 {
    let pool_client =
        pool::Client::new(
            e,
            blend_pool,
        );

    let reserve =
        pool_client.get_reserve(asset);

    let positions =
        pool_client.get_positions(
            vault_address,
        );

    let reserve_index =
        reserve.config.index;

    let b_tokens =
        positions
            .collateral
            .get(reserve_index)
            .unwrap_or(0);

    b_tokens_to_assets(
        b_tokens,
        reserve.data.b_rate,
        reserve.scalar,
    )
}

// =====================================================
// CONVERT BLEND bTOKENS TO UNDERLYING ASSETS
// =====================================================

pub fn b_tokens_to_assets(
    b_tokens: i128,
    b_rate: i128,
    scalar: i128,
) -> i128 {
    if b_tokens <= 0 {
        return 0;
    }

    if scalar <= 0 {
        return 0;
    }

    b_tokens
        .checked_mul(b_rate)
        .unwrap_or(0)
        / scalar
}