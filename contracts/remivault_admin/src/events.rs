use soroban_sdk::{
    contracttype,
    Address,
    Env,
};

// =====================================================
// ADMIN EVENT ENUM
// =====================================================

#[contracttype]
#[derive(Clone)]
pub enum AdminEvent {
    HarvestExecuted(i128, i128, u64),
    FeeCollected(i128, Address),
    VaultStatePaused(u32, soroban_sdk::String),
    VaultStateUnpaused(u64),
    FeeRateUpdated(i128, i128),
    TreasuryAddressUpdated(Address, Address),
    BlendPoolMigrated(Address, Address),
}

// =====================================================
// EMIT HARVEST EVENT
// =====================================================

pub fn emit_harvest_executed(
    e: &Env,
    gross_yield: i128,
    platform_fees: i128,
) {

    let event = AdminEvent::HarvestExecuted(
        gross_yield,
        platform_fees,
        e.ledger().timestamp(),
    );

    e.events().publish(
        ("harvest_executed",),
        event,
    );
}

// =====================================================
// EMIT FEE COLLECTION EVENT
// =====================================================

pub fn emit_fee_collected(
    e: &Env,
    amount: i128,
    treasury: Address,
) {

    let event = AdminEvent::FeeCollected(
        amount,
        treasury,
    );

    e.events().publish(
        ("fee_collected",),
        event,
    );
}

// =====================================================
// EMIT VAULT PAUSED EVENT
// =====================================================

pub fn emit_vault_paused(
    e: &Env,
    pause_level: u32,
    reason: soroban_sdk::String,
) {

    let event = AdminEvent::VaultStatePaused(
        pause_level,
        reason,
    );

    e.events().publish(
        ("vault_paused",),
        event,
    );
}

// =====================================================
// EMIT VAULT UNPAUSED EVENT
// =====================================================

pub fn emit_vault_unpaused(
    e: &Env,
) {

    let event =
        AdminEvent::VaultStateUnpaused(
            e.ledger().timestamp(),
        );

    e.events().publish(
        ("vault_unpaused",),
        event,
    );
}

// =====================================================
// EMIT FEE RATE UPDATE EVENT
// =====================================================

pub fn emit_fee_rate_updated(
    e: &Env,
    old_fee_bps: i128,
    new_fee_bps: i128,
) {

    let event =
        AdminEvent::FeeRateUpdated(
            old_fee_bps,
            new_fee_bps,
        );

    e.events().publish(
        ("fee_rate_updated",),
        event,
    );
}

// =====================================================
// EMIT TREASURY UPDATE EVENT
// =====================================================

pub fn emit_treasury_updated(
    e: &Env,
    old_treasury: Address,
    new_treasury: Address,
) {

    let event =
        AdminEvent::TreasuryAddressUpdated(
            old_treasury,
            new_treasury,
        );

    e.events().publish(
        ("treasury_updated",),
        event,
    );
}

// =====================================================
// EMIT BLEND POOL MIGRATION EVENT
// =====================================================

pub fn emit_blend_pool_migrated(
    e: &Env,
    old_pool: Address,
    new_pool: Address,
) {

    let event =
        AdminEvent::BlendPoolMigrated(
            old_pool,
            new_pool,
        );

    e.events().publish(
        ("blend_pool_migrated",),
        event,
    );
}