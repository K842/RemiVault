use soroban_sdk::contracterror;

// =====================================================
// CONTRACT ERROR TYPES
// =====================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {

    // =================================================
    // INITIALIZATION ERRORS
    // =================================================

    // Contract already initialized
    AlreadyInitialized = 1,

    // Vault not initialized yet
    NotInitialized = 2,

    // =================================================
    // USER BALANCE ERRORS
    // =================================================

    // User lacks enough shares
    InsufficientShares = 3,

    // User lacks enough balance
    InsufficientBalance = 4,

    // =================================================
    // CONTRACT PAUSE ERRORS
    // =================================================

    // Full vault pause
    VaultPaused = 5,

    // Withdrawals paused
    WithdrawalsPaused = 6,

    // Deposits paused
    DepositsPaused = 7,

    // =================================================
    // AUTHORIZATION ERRORS
    // =================================================

    // Unauthorized caller
    Unauthorized = 8,

    // =================================================
    // MATH & LIMIT ERRORS
    // =================================================

    // Invalid amount
    InvalidAmount = 9,

    // Overflow/underflow issue
    MathOverflow = 10,

    // Division by zero
    DivisionByZero = 11,

    // =================================================
    // BLEND PROTOCOL ERRORS
    // =================================================

    // Blend rejected request
    BlendRejected = 12,

    // Blend liquidity unavailable
    BlendLiquidityUnavailable = 13,

    // Reserve sync failure
    ReserveSyncFailed = 14,
}