use soroban_sdk::{
    contract,
    contractimpl,
    Address,
    Env,
    Symbol,
    IntoVal,
};

use crate::storage::{
    DataKey,
    get_admin,
    set_admin,
    get_core_vault,
    set_core_vault,
    get_asset,
    set_asset,
    get_fee_rate,
    set_fee_rate,
    get_treasury,
    set_treasury,
    set_emergency_state,
    EmergencyState,
    get_blend_pool,
    set_blend_pool,
    get_last_harvest_assets,
    set_last_harvest_assets,
};

use crate::events;
use crate::blend_client;
use crate::error::AdminError;

// =====================================================
// ADMIN CONTRACT
// =====================================================

#[contract]

pub struct RemiVaultAdmin;

// =====================================================
// CONTRACT IMPLEMENTATION
// =====================================================

#[contractimpl]
impl RemiVaultAdmin {

    pub fn initialize(
        e: Env,
        admin: Address,
        core_vault: Address,
        asset: Address,
        treasury: Address,
        blend_pool: Address,
        fee_rate_bps: i128,
    ) {

        // Prevent reinitialization
        if e.storage()
            .instance()
            .has(&DataKey::AdminAddress)
        {
            panic!("already initialized");
        }

        // Require admin auth
        admin.require_auth();

        // Validate fee rate
        if fee_rate_bps < 0 || fee_rate_bps > 5_000 {
            panic!("invalid fee rate");
        }

        // Store admin
        set_admin(
            &e,
            &admin,
        );

        // Store core vault
        set_core_vault(
            &e,
            &core_vault,
        );

        // Store asset
        set_asset(
            &e,
            &asset,
        );

        // Store treasury
        set_treasury(
            &e,
            &treasury,
        );

        // Store blend pool
        set_blend_pool(
            &e,
            &blend_pool,
        );

        // Store fee rate
        set_fee_rate(
            &e,
            &fee_rate_bps,
        );

        // Set default emergency state
        set_emergency_state(
            &e,
            &EmergencyState::Active,
        );

        // Initialize harvest checkpoint at 0
        set_last_harvest_assets(
            &e,
            &0,
        );
    }


fn get_validated_core_vault(
    e: &Env,
) -> Address {

    let vault =
        get_core_vault(e);

    // Address type already validated by Soroban
    vault
}


fn verify_admin(
    e: &Env,
) -> Result<(), AdminError> {

    let admin =
        get_admin(e);

    admin.require_auth();

    Ok(())
}
pub fn harvest(
    e: Env,
) -> Result<(), AdminError> {

    // Admin auth
    Self::verify_admin(&e)?;

    

    // Validate vault
    Self::get_validated_core_vault(&e);

    // Get Blend pool address
    let blend_pool =
        get_blend_pool(&e);

    // Get core vault address
    let core_vault =
        Self::get_validated_core_vault(&e);

    // Get asset address
    let asset =
        get_asset(&e);

    // Query REAL reserve value
    let current_assets =
        blend_client::get_total_managed_assets(
            &e,
            &blend_pool,
            &core_vault,
             &asset,
        );

    // Load REAL checkpoint
    let previous_assets =
        get_last_harvest_assets(&e);

    // No profit generated
    if current_assets <= previous_assets {
        return Ok(());
    }

    // Gross protocol yield
    let gross_yield =
        current_assets
            .checked_sub(previous_assets)
            .ok_or(
                AdminError::MathOverflow
            )?;

    // Fee rate
    let fee_bps =
        get_fee_rate(&e);

    // Platform fee
    let platform_fee =
        gross_yield
            .checked_mul(fee_bps)
            .ok_or(
                AdminError::MathOverflow
            )?
            / 10_000;

    // Net user yield
    let net_yield =
        gross_yield
            .checked_sub(platform_fee)
            .ok_or(
                AdminError::MathOverflow
            )?;

    // Cross-contract bridge function
    Self::_harvest_internal(
    &e,
    net_yield,
    platform_fee,
)?;

    // Save NEW checkpoint
    set_last_harvest_assets(
        &e,
        &current_assets,
    );

    // Emit audit event
    events::emit_harvest_executed(
        &e,
        gross_yield,
        platform_fee,
    );

    Ok(())
}



pub fn collect_fees(
    e: Env,
) -> Result<(), AdminError> {

    // =================================================
    // VERIFY ADMIN AUTHORIZATION
    // =================================================

    Self::verify_admin(&e)?;

    // =================================================
    // LOAD TREASURY ADDRESS
    // =================================================

    let treasury =
        get_treasury(&e);

    // =================================================
    // CALL BRIDGE HELPER TO COLLECT FEES
    // =================================================

    let collected_fees =
        Self::call_collect_fees_internal(
            &e,
            treasury.clone(),
        )?;

    // =================================================
    // EMIT AUDIT EVENT
    // =================================================

    events::emit_fee_collected(
        &e,
        collected_fees,
        treasury,
    );

    Ok(())
}



pub fn pause(
    e: Env,
    level: u32,
) -> Result<(), AdminError> {

    // =================================================
    // VERIFY ADMIN AUTHORIZATION
    // =================================================

    Self::verify_admin(&e)?;
    Self::validate_pause_level(level)?;

    // =================================================
    // VALIDATE CORE VAULT
    // =================================================

    let _core_vault =
        Self::get_validated_core_vault(&e);

    // =================================================
    // UPDATE EMERGENCY STATE
    // =================================================

    set_emergency_state(
        &e,
        &EmergencyState::Paused,
    );

    // =================================================
    // EXECUTE BRIDGE CALL
    // =================================================

    Self::call_pause_internal(
        &e,
        level,
    )?;

    // =================================================
    // EMIT EVENT
    // =================================================

    events::emit_vault_paused(
        &e,
        level,
        soroban_sdk::String::from_str(
            &e,
            "Emergency pause activated",
        ),
    );

    Ok(())
}

// =====================================================
// UNPAUSE VAULT
// =====================================================

pub fn unpause(
    e: Env,
) -> Result<(), AdminError> {

    // =================================================
    // VERIFY ADMIN AUTHORIZATION
    // =================================================

    Self::verify_admin(&e)?;

    // =================================================
    // VALIDATE CORE VAULT
    // =================================================

    let _core_vault =
        Self::get_validated_core_vault(&e);

    // =================================================
    // RESTORE ACTIVE STATE
    // =================================================

    set_emergency_state(
        &e,
        &EmergencyState::Active,
    );

    // =================================================
    // EXECUTE BRIDGE CALL
    // =================================================

    Self::call_unpause_internal(&e)?;

    // =================================================
    // EMIT EVENT
    // =================================================

    events::emit_vault_unpaused(
        &e,
    );

    Ok(())
}



pub fn update_fee(
    e: Env,
    new_fee_bps: i128,
) -> Result<(), AdminError> {

    // =================================================
    // VERIFY ADMIN AUTHORIZATION
    // =================================================

    Self::verify_admin(&e)?;

    // =================================================
    // VALIDATE FEE LIMIT
    // =================================================

    Self::validate_fee_rate(new_fee_bps)?;

    // =================================================
    // LOAD OLD FEE RATE
    // =================================================

    let old_fee_bps =
        get_fee_rate(&e);

    // =================================================
    // STORE NEW FEE RATE
    // =================================================

    set_fee_rate(
        &e,
        &new_fee_bps,
    );

    // =================================================
    // EXECUTE BRIDGE CALL
    // =================================================

    Self::call_update_fee_internal(
        &e,
        new_fee_bps,
    )?;

    // =================================================
    // EMIT EVENT
    // =================================================

    events::emit_fee_rate_updated(
        &e,
        old_fee_bps,
        new_fee_bps,
    );

    Ok(())
}

// =====================================================
// UPDATE TREASURY ADDRESS
// =====================================================

pub fn update_treasury(
    e: Env,
    new_treasury: Address,
) -> Result<(), AdminError> {

    // =================================================
    // VERIFY ADMIN AUTHORIZATION
    // =================================================

    Self::verify_admin(&e)?;

    // =================================================
    // LOAD OLD TREASURY
    // =================================================

    let old_treasury =
        get_treasury(&e);

    // =================================================
    // STORE NEW TREASURY
    // =================================================

    set_treasury(
        &e,
        &new_treasury,
    );

    // =================================================
    // EXECUTE BRIDGE CALL
    // =================================================

    Self::call_update_treasury_internal(
        &e,
        new_treasury.clone(),
    )?;

    // =================================================
    // EMIT EVENT
    // =================================================

    events::emit_treasury_updated(
        &e,
        old_treasury,
        new_treasury,
    );

    Ok(())
}

// =====================================================
// MIGRATE BLEND POOL
// =====================================================

pub fn migrate_blend_pool(
    e: Env,
    new_pool: Address,
) -> Result<(), AdminError> {

    // =================================================
    // VERIFY ADMIN AUTHORIZATION
    // =================================================

    Self::verify_admin(&e)?;

    // =================================================
    // LOAD OLD POOL
    // =================================================

    let old_pool =
        get_blend_pool(&e);

    // =================================================
    // STORE NEW POOL
    // =================================================

    set_blend_pool(
        &e,
        &new_pool,
    );

    // =================================================
    // EXECUTE BRIDGE CALL
    // =================================================

    Self::call_migrate_blend_pool_internal(
        &e,
        new_pool.clone(),
    )?;

    // =================================================
    // EMIT EVENT
    // =================================================

    events::emit_blend_pool_migrated(
        &e,
        old_pool,
        new_pool,
    );

    Ok(())
}

// =====================================================
// VALIDATION FUNCTIONS
// =====================================================

fn validate_pause_level(
    level: u32,
) -> Result<(), AdminError> {

    // Allowed levels:
    // 0 = active
    // 1 = deposit pause
    // 2 = full emergency

    if level > 2 {
        return Err(
            AdminError::InvalidPauseLevel
        );
    }

    Ok(())
}

fn validate_fee_rate(
    fee_bps: i128,
) -> Result<(), AdminError> {

    // Reject negative fees
    if fee_bps < 0 {
        return Err(
            AdminError::InvalidFeeRate
        );
    }

    // Reject fees exceeding 50%
    if fee_bps > 5_000 {
        return Err(
            AdminError::FeeRateExceedsLimit
        );
    }

    Ok(())
}

// =====================================================
// INTERNAL HARVEST BRIDGE
// =====================================================

fn _harvest_internal(
    e: &Env,
    net_yield: i128,
    platform_fee: i128,
) -> Result<(), AdminError> {

    // ================================================
    // VALIDATE CORE VAULT
    // ================================================

    let core_vault =
        Self::get_validated_core_vault(e);

    // ================================================
    // BUILD FUNCTION NAME
    // ================================================

    let fn_name =
        Symbol::new(
            e,
            "_harvest_internal",
        );

    // ================================================
    // BUILD ARGUMENT PACKET
    // ================================================

    let args = (
        net_yield,
        platform_fee,
    )
        .into_val(e);

    // ================================================
    // EXECUTE CROSS-CONTRACT CALL
    // ================================================

    e.invoke_contract::<()>(
        &core_vault,
        &fn_name,
        args,
    );

    Ok(())
}

// =====================================================
// INTERNAL FEE COLLECTION BRIDGE
// =====================================================

fn call_collect_fees_internal(
    e: &Env,
    treasury: Address,
) -> Result<i128, AdminError> {

    // ================================================
    // VALIDATE CORE VAULT
    // ================================================

    let core_vault =
        Self::get_validated_core_vault(e);

    // ================================================
    // BUILD FUNCTION NAME
    // ================================================

    let fn_name =
        Symbol::new(
            e,
            "_collect_fees_internal",
        );

    // ================================================
    // BUILD ARGUMENT PACKET
    // ================================================

    let args = (
        treasury,
    )
        .into_val(e);

    // ================================================
    // EXECUTE CROSS-CONTRACT CALL
    // ================================================

    let collected_fees: i128 =
        e.invoke_contract(
            &core_vault,
            &fn_name,
            args,
        );

    Ok(collected_fees)
}

// =====================================================
// INTERNAL PAUSE BRIDGE
// =====================================================

fn call_pause_internal(
    e: &Env,
    level: u32,
) -> Result<(), AdminError> {

    // ================================================
    // VALIDATE CORE VAULT
    // ================================================

    let core_vault =
        Self::get_validated_core_vault(e);

    // ================================================
    // BUILD FUNCTION NAME
    // ================================================

    let fn_name =
        Symbol::new(
            e,
            "_pause_internal",
        );

    // ================================================
    // BUILD ARGUMENT PACKET
    // ================================================

    let args = (
        level,
    )
        .into_val(e);

    // ================================================
    // EXECUTE CROSS-CONTRACT CALL
    // ================================================

    e.invoke_contract::<()>(
        &core_vault,
        &fn_name,
        args,
    );

    Ok(())
}

// =====================================================
// INTERNAL UNPAUSE BRIDGE
// =====================================================

fn call_unpause_internal(
    e: &Env,
) -> Result<(), AdminError> {

    // ================================================
    // VALIDATE CORE VAULT
    // ================================================

    let core_vault =
        Self::get_validated_core_vault(e);

    // ================================================
    // BUILD FUNCTION NAME
    // ================================================

    let fn_name =
        Symbol::new(
            e,
            "_unpause_internal",
        );

    // ================================================
    // EMPTY ARGUMENT PACKET
    // ================================================

    let args = ()
        .into_val(e);

    // ================================================
    // EXECUTE CROSS-CONTRACT CALL
    // ================================================

    e.invoke_contract::<()>(
        &core_vault,
        &fn_name,
        args,
    );

    Ok(())
}

// =====================================================
// INTERNAL FEE UPDATE BRIDGE
// =====================================================

fn call_update_fee_internal(
    e: &Env,
    new_fee_bps: i128,
) -> Result<(), AdminError> {

    // ================================================
    // VALIDATE CORE VAULT
    // ================================================

    let core_vault =
        Self::get_validated_core_vault(e);

    // ================================================
    // BUILD FUNCTION NAME
    // ================================================

    let fn_name =
        Symbol::new(
            e,
            "_update_fee_internal",
        );

    // ================================================
    // BUILD ARGUMENT PACKET
    // ================================================

    let args = (
        new_fee_bps,
    )
        .into_val(e);

    // ================================================
    // EXECUTE CROSS-CONTRACT CALL
    // ================================================

    e.invoke_contract::<()>(
        &core_vault,
        &fn_name,
        args,
    );

    Ok(())
}

// =====================================================
// INTERNAL TREASURY UPDATE BRIDGE
// =====================================================

fn call_update_treasury_internal(
    e: &Env,
    treasury: Address,
) -> Result<(), AdminError> {

    // ================================================
    // VALIDATE CORE VAULT
    // ================================================

    let core_vault =
        Self::get_validated_core_vault(e);

    // ================================================
    // BUILD FUNCTION NAME
    // ================================================

    let fn_name =
        Symbol::new(
            e,
            "_update_treasury_internal",
        );

    // ================================================
    // BUILD ARGUMENT PACKET
    // ================================================

    let args = (
        treasury,
    )
        .into_val(e);

    // ================================================
    // EXECUTE CROSS-CONTRACT CALL
    // ================================================

    e.invoke_contract::<()>(
        &core_vault,
        &fn_name,
        args,
    );

    Ok(())
}

// =====================================================
// INTERNAL BLEND POOL MIGRATION BRIDGE
// =====================================================

fn call_migrate_blend_pool_internal(
    e: &Env,
    new_pool: Address,
) -> Result<(), AdminError> {

    // ================================================
    // VALIDATE CORE VAULT
    // ================================================

    let core_vault =
        Self::get_validated_core_vault(e);

    // ================================================
    // BUILD FUNCTION NAME
    // ================================================

    let fn_name =
        Symbol::new(
            e,
            "_migrate_blend_pool_internal",
        );

    // ================================================
    // BUILD ARGUMENT PACKET
    // ================================================

    let args = (
        new_pool,
    )
        .into_val(e);

    // ================================================
    // EXECUTE CROSS-CONTRACT CALL
    // ================================================

    e.invoke_contract::<()>(
        &core_vault,
        &fn_name,
        args,
    );

    Ok(())
}
}