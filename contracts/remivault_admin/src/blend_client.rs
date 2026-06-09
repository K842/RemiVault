use soroban_sdk::{
    Address,
    Env,
};

use blend_contract_sdk::pool;

// =====================================================
// READ CORE VAULT ASSETS MANAGED IN BLEND
// =====================================================

pub fn get_total_managed_assets(
    e: &Env,
    blend_pool: &Address,
    core_vault: &Address,
    asset: &Address,
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
            core_vault,
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
// CONVERT bTOKENS TO UNDERLYING ASSETS
// =====================================================

fn b_tokens_to_assets(
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