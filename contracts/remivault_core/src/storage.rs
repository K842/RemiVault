use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Config,
    VaultState,
    PauseLevel,
    FeeRateBps,
    UserShares(Address),
    Allowance(Address,Address)
}

#[contracttype]
#[derive(Clone)]
pub struct Config {
    pub asset: Address,
    pub admin: Address,
    pub treasury: Address,
    pub blend_pool: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct VaultState {
    pub total_shares: i128,
    pub total_assets: i128,
    pub accrued_fees: i128,
    pub last_reserve_brate: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum PauseLevel {
    None,
    DepositOnly,
    EmergencyPause,
}