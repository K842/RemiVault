// =====================================================
// INTERNAL PRECISION CONSTANTS
// =====================================================

// USDC = 6 decimals
// Internal vault accounting = 18 decimals
// Difference = 12 decimals
pub const INTERNAL_SCALE: i128 = 1_000_000_000_000; // 10^12

// Virtual offset protection
pub const VIRTUAL_ASSETS: i128 = 1_000_000;
pub const VIRTUAL_SHARES: i128 = 1_000_000;

// =====================================================
// SCALE UP
// Convert 6-decimal USDC → 18-decimal precision
// =====================================================

pub fn scale_up(
    amount: i128,
) -> i128 {

    amount
        .checked_mul(INTERNAL_SCALE)
        .unwrap()
}

// =====================================================
// SCALE DOWN
// Convert 18-decimal precision → 6-decimal USDC
// =====================================================

pub fn scale_down(
    amount: i128,
) -> i128 {

    amount / INTERNAL_SCALE
}

// =====================================================
// ROUND DOWN
// Used during deposits
// Protects vault from over-minting shares
// =====================================================

pub fn mul_div_down(
    a: i128,
    b: i128,
    denominator: i128,
) -> i128 {

    a.checked_mul(b)
        .unwrap()
        / denominator
}

// =====================================================
// ROUND UP
// Used during withdrawals
// Protects vault from under-burning shares
// =====================================================

pub fn mul_div_up(
    a: i128,
    b: i128,
    denominator: i128,
) -> i128 {

    let product =
        a.checked_mul(b)
            .unwrap();

    let mut result =
        product / denominator;

    // If remainder exists:
    // round upward
    if product % denominator != 0 {
        result += 1;
    }

    result
}

// =====================================================
// CONVERT ASSETS → SHARES
// ROUND DOWN FOR SAFETY
// =====================================================

pub fn convert_to_shares(
    assets: i128,
    total_assets: i128,
    total_shares: i128,
) -> i128 {
    mul_div_down(
        assets,
        total_shares + VIRTUAL_SHARES,
        total_assets + VIRTUAL_ASSETS,
    )
}

// =====================================================
// CONVERT SHARES → ASSETS
// ROUND UP FOR SAFETY
// =====================================================

pub fn convert_to_assets(
    shares: i128,
    total_assets: i128,
    total_shares: i128,
) -> i128 {
    mul_div_up(
        shares,
        total_assets + VIRTUAL_ASSETS,
        total_shares + VIRTUAL_SHARES,
    )
}