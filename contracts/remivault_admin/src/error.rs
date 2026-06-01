use soroban_sdk::contracterror;

// =====================================================
// ADMIN CONTRACT ERRORS
// =====================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AdminError {

    // =================================================
    // AUTHORIZATION ERRORS
    // =================================================

    // Non-admin attempted privileged action
    NotAdmin = 1,

    // =================================================
    // CORE VAULT VALIDATION ERRORS
    // =================================================

    // Core vault address missing or invalid
    InvalidCoreAddress = 2,

    // =================================================
    // FEE CONFIGURATION ERRORS
    // =================================================

    // Fee exceeds maximum allowed limit
    // Max allowed = 5000 bps = 50%
    FeeRateExceedsLimit = 3,

    // =================================================
    // CROSS-CONTRACT EXECUTION ERRORS
    // =================================================

    // Downstream core vault call failed
    CrossContractCallFailed = 4,

    // =================================================
    // MATH ERRORS
    // =================================================

    // Arithmetic overflow detected
    MathOverflow = 5,

    // =================================================
    // VALIDATION ERRORS
    // =================================================

    // Invalid pause/emergency level
    InvalidPauseLevel = 6,

    // Fee rate is invalid or negative
    InvalidFeeRate = 7,

    // Amount is invalid or non-positive
    InvalidAmount = 8,

    // Contract is paused
    ContractPaused = 9,

    // Address is invalid
    InvalidAddress = 10,

    // No yield available for harvest
    NoYieldAvailable = 11,
}