use soroban_sdk::{
    contracttype,
    Address,
    Env,
};

// =====================================================
// STORAGE KEYS
// =====================================================

#[contracttype]
#[derive(Clone)]
pub enum DataKey {

    // Main admin authority
    AdminAddress,

    // Core vault contract address
    CoreVaultAddress,

    // Emergency pause status
    EmergencyPauseState,

    // Treasury address
    TreasuryAddress,

    // Fee rate basis points
    FeeRateBps,

    // Last harvest checkpoint
    LastHarvestAssets,

    // Blend pool address
    BlendPoolAddress,
}

// =====================================================
// EMERGENCY PAUSE STATE
// =====================================================

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum EmergencyState {

    // Vault active
    Active,

    // Vault paused
    Paused,
}

// =====================================================
// ADMIN CONFIG STRUCTURE
// =====================================================

#[contracttype]
#[derive(Clone)]
pub struct AdminConfig {

    // Main protocol admin
    pub admin: Address,

    // Core vault contract
    pub core_vault: Address,

    // Treasury receiver
    pub treasury: Address,

    // Protocol fee rate
    pub fee_rate_bps: i128,
}

// =====================================================
// GET ADMIN ADDRESS
// =====================================================

pub fn get_admin(
    e: &Env,
) -> Address {

    e.storage()
        .instance()
        .get(&DataKey::AdminAddress)
        .unwrap()
}

// =====================================================
// SET ADMIN ADDRESS
// =====================================================

pub fn set_admin(
    e: &Env,
    admin: &Address,
) {

    e.storage()
        .instance()
        .set(
            &DataKey::AdminAddress,
            admin,
        );
}

// =====================================================
// GET CORE VAULT ADDRESS
// =====================================================

pub fn get_core_vault(
    e: &Env,
) -> Address {

    e.storage()
        .instance()
        .get(&DataKey::CoreVaultAddress)
        .unwrap()
}

// =====================================================
// SET CORE VAULT ADDRESS
// =====================================================

pub fn set_core_vault(
    e: &Env,
    vault: &Address,
) {

    e.storage()
        .instance()
        .set(
            &DataKey::CoreVaultAddress,
            vault,
        );
}

// =====================================================
// GET TREASURY ADDRESS
// =====================================================

pub fn get_treasury(
    e: &Env,
) -> Address {

    e.storage()
        .instance()
        .get(&DataKey::TreasuryAddress)
        .unwrap()
}

// =====================================================
// SET TREASURY ADDRESS
// =====================================================

pub fn set_treasury(
    e: &Env,
    treasury: &Address,
) {

    e.storage()
        .instance()
        .set(
            &DataKey::TreasuryAddress,
            treasury,
        );
}

// =====================================================
// GET FEE RATE
// =====================================================

pub fn get_fee_rate(
    e: &Env,
) -> i128 {

    e.storage()
        .instance()
        .get(&DataKey::FeeRateBps)
        .unwrap_or(0)
}

// =====================================================
// SET FEE RATE
// =====================================================

pub fn set_fee_rate(
    e: &Env,
    fee_rate: &i128,
) {

    e.storage()
        .instance()
        .set(
            &DataKey::FeeRateBps,
            fee_rate,
        );
}

// =====================================================
// GET BLEND POOL
// =====================================================

pub fn get_blend_pool(
    e: &Env,
) -> Address {

    e.storage()
        .instance()
        .get(&DataKey::BlendPoolAddress)
        .unwrap()
}

// =====================================================
// SET BLEND POOL
// =====================================================

pub fn set_blend_pool(
    e: &Env,
    pool: &Address,
) {

    e.storage()
        .instance()
        .set(
            &DataKey::BlendPoolAddress,
            pool,
        );
}

// =====================================================
// GET EMERGENCY STATE
// =====================================================


pub fn get_emergency_state(
    e: &Env,
) -> EmergencyState {

    e.storage()
        .instance()
        .get(&DataKey::EmergencyPauseState)
        .unwrap_or(
            EmergencyState::Active
        )
}

// =====================================================
// SET EMERGENCY STATE
// =====================================================

pub fn set_emergency_state(
    e: &Env,
    state: &EmergencyState,
) {

    e.storage()
        .instance()
        .set(
            &DataKey::EmergencyPauseState,
            state,
        );
}

// =====================================================
// GET LAST HARVEST CHECKPOINT
// =====================================================

pub fn get_last_harvest_assets(
    e: &Env,
) -> i128 {

    e.storage()
        .instance()
        .get(&DataKey::LastHarvestAssets)
        .unwrap_or(0)
}

// =====================================================
// SET LAST HARVEST CHECKPOINT
// =====================================================

pub fn set_last_harvest_assets(
    e: &Env,
    amount: &i128,
) {

    e.storage()
        .instance()
        .set(
            &DataKey::LastHarvestAssets,
            amount,
        );
}