use soroban_sdk::{
    Address,
    Env,
};

// =====================================================
// INITIALIZATION EVENT
// =====================================================

pub fn emit_initialize(
    e: &Env,
    admin: Address,
    asset: Address,
    treasury: Address,
) {

    e.events().publish(
        ("initialize", admin),
        (asset, treasury),
    );
}

// =====================================================
// DEPOSIT EVENT
// =====================================================

pub fn emit_deposit(
    e: &Env,
    user: Address,
    assets: i128,
    shares: i128,
) {

    e.events().publish(
        ("deposit", user),
        (assets, shares),
    );
}

// =====================================================
// WITHDRAW EVENT
// =====================================================

pub fn emit_withdraw(
    e: &Env,
    user: Address,
    assets: i128,
    shares: i128,
) {

    e.events().publish(
        ("withdraw", user),
        (assets, shares),
    );
}

// =====================================================
// HARVEST EVENT
// =====================================================

pub fn emit_harvest(
    e: &Env,
    harvested_assets: i128,
) {

    e.events().publish(
        ("harvest",),
        harvested_assets,
    );
}

// =====================================================
// FEE COLLECTION EVENT
// =====================================================

pub fn emit_fee_collection(
    e: &Env,
    fee_amount: i128,
) {

    e.events().publish(
        ("fee_collected",),
        fee_amount,
    );
}

// =====================================================
// RESERVE SNAPSHOT EVENT
// =====================================================

pub fn emit_reserve_snapshot(
    e: &Env,
    total_assets: i128,
    total_shares: i128,
    reserve_balance: i128,
) {

    e.events().publish(
        ("reserve_snapshot",),
        (
            total_assets,
            total_shares,
            reserve_balance,
        ),
    );
}

// =====================================================
// PAUSE EVENT
// =====================================================

pub fn emit_pause_update(
    e: &Env,
    level: u32,
) {

    e.events().publish(
        ("pause_level_updated",),
        level,
    );
}