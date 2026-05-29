use soroban_sdk::{
    contractclient,
    Address,
    Env,
    Vec,
};

// =====================================================
// BLEND POSITION STRUCTURE
// Represents vault reserve ownership inside Blend
// =====================================================

#[derive(Clone)]
#[contracttype]
pub struct PoolPosition {

    // Total supplied assets
    pub supplied: i128,

    // Borrowed amount
    pub borrowed: i128,

    // Collateral amount
    pub collateral: i128,
}

// =====================================================
// BLEND RATE STRUCTURE
// Represents lending APY information
// =====================================================

#[derive(Clone)]
#[contracttype]
pub struct RateData {

    // Supply APY
    pub supply_rate_bps: i128,

    // Borrow APY
    pub borrow_rate_bps: i128,
}

// =====================================================
// BLEND REQUEST TYPE
// Used during submit() pipeline
// =====================================================

#[derive(Clone)]
#[contracttype]
pub struct Request {

    // Request type
    // Example:
    // 0 = supply
    // 1 = withdraw
    pub request_type: u32,

    // Asset address
    pub asset: Address,

    // Amount
    pub amount: i128,
}

// =====================================================
// BLEND CLIENT INTERFACE
// External protocol interface
// =====================================================

#[contractclient(name = "BlendPoolClient")]
pub trait BlendPoolTrait {

    // =================================================
    // SUBMIT PIPELINE
    // =================================================

    fn submit(
        env: Env,
        from: Address,
        spender: Address,
        to: Address,
        requests: Vec<Request>,
    );

    // =================================================
    // POSITION QUERY
    // =================================================

    fn get_position(
        env: Env,
        user: Address,
    ) -> PoolPosition;

    // =================================================
    // RATE QUERY
    // =================================================

    fn get_rates(
        env: Env,
        asset: Address,
    ) -> RateData;
}