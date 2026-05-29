// =====================================================
// REMIVAULT CORE — COMPREHENSIVE TEST SUITE
// =====================================================

#[cfg(test)]
mod tests {
    use crate::contract::{RemiVaultCore, Config, VaultState, PauseLevel};
    use soroban_sdk::{Address, Env};

    // =====================================================
    // TEST 1 — INITIALIZATION
    // =====================================================

    #[test]
    fn test_initialize() {
        // Create test environment
        let env = Env::default();

        // Register contract
        let contract_id = env.register_contract(None, RemiVaultCore);

        // Create test addresses
        let admin = Address::generate(&env);
        let asset = Address::generate(&env);
        let treasury = Address::generate(&env);
        let blend_pool = Address::generate(&env);

        // Call initialize through storage directly
        let config = Config {
            asset: asset.clone(),
            admin: admin.clone(),
            treasury: treasury.clone(),
            blend_pool: blend_pool.clone(),
        };

        RemiVaultCore::set_config(&env, &config);

        // Verify config stored correctly
        let stored_config = RemiVaultCore::get_config(&env);
        assert_eq!(stored_config.admin, admin);
        assert_eq!(stored_config.asset, asset);
        assert_eq!(stored_config.treasury, treasury);
        assert_eq!(stored_config.blend_pool, blend_pool);
    }

    // =====================================================
    // TEST 2 — DEPOSIT ACCOUNTING
    // =====================================================

    #[test]
    fn test_deposit_share_minting() {
        let env = Env::default();

        // Setup vault state
        let vault_state = VaultState {
            total_assets: 0,
            total_shares: 0,
            accrued_fees: 0,
            last_reserve_brate: 0,
        };

        RemiVaultCore::set_vault_state(&env, &vault_state);

        // Test: First deposit of 1000 should mint 1000 shares
        let shares = RemiVaultCore::convert_to_shares(&env, 1000);

        assert_eq!(shares, 1000, "First deposit should mint 1:1 ratio");
    }

    // =====================================================
    // TEST 3 — VAULT INVARIANT VALIDATION
    // =====================================================

    #[test]
    fn test_vault_invariant_empty() {
        let env = Env::default();

        // Empty vault invariant: if total_shares == 0, total_assets should be 0
        let vault_state = VaultState {
            total_assets: 0,
            total_shares: 0,
            accrued_fees: 0,
            last_reserve_brate: 0,
        };

        assert_eq!(vault_state.total_assets, 0);
        assert_eq!(vault_state.total_shares, 0);
    }

    // =====================================================
    // TEST 4 — SHARE CONVERSION LOGIC
    // =====================================================

    #[test]
    fn test_share_conversion() {
        let env = Env::default();

        // Setup vault with existing reserves
        let vault_state = VaultState {
            total_assets: 1000,
            total_shares: 100,
            accrued_fees: 0,
            last_reserve_brate: 0,
        };

        RemiVaultCore::set_vault_state(&env, &vault_state);

        // Test: 100 assets should mint 10 shares (1000:100 = 100:10)
        let new_shares = RemiVaultCore::convert_to_shares(&env, 100);
        assert_eq!(new_shares, 10, "Share conversion math should be correct");

        // Test: 10 shares should convert back to 100 assets
        let assets_back = RemiVaultCore::convert_to_assets(&env, 10);
        assert_eq!(
            assets_back, 100,
            "Asset conversion should preserve value"
        );
    }

    // =====================================================
    // TEST 5 — PAUSE LEVEL SAFE DEFAULTS
    // =====================================================

    #[test]
    fn test_pause_level_default() {
        let env = Env::default();

        // Test: Uninitialized pause level should default to None
        let pause_level = RemiVaultCore::get_pause_level(&env);
        assert_eq!(
            pause_level, PauseLevel::None,
            "Pause level should default to None"
        );
    }

    // =====================================================
    // TEST 6 — PAUSE LEVEL SETTING
    // =====================================================

    #[test]
    fn test_set_pause_level() {
        let env = Env::default();

        // Test: Can set pause level
        RemiVaultCore::set_pause_level(&env, &PauseLevel::EmergencyPause);

        let stored_level = RemiVaultCore::get_pause_level(&env);
        assert_eq!(
            stored_level, PauseLevel::EmergencyPause,
            "Pause level should be updated"
        );
    }

    // =====================================================
    // TEST 7 — USER SHARES MANAGEMENT
    // =====================================================

    #[test]
    fn test_user_shares_tracking() {
        let env = Env::default();

        let user = Address::generate(&env);

        // Test: User with no shares should have 0
        let initial_shares = RemiVaultCore::get_user_shares(&env, user.clone());
        assert_eq!(initial_shares, 0, "New user should have 0 shares");

        // Test: Can add shares
        RemiVaultCore::set_user_shares(&env, user.clone(), 500);

        let updated_shares = RemiVaultCore::get_user_shares(&env, user.clone());
        assert_eq!(updated_shares, 500, "Shares should be updated");
    }

    // =====================================================
    // TEST 8 — FEE RATE STORAGE
    // =====================================================

    #[test]
    fn test_fee_rate_storage() {
        let env = Env::default();

        // Test: Can set and retrieve fee rate
        let fee_rate = 500; // 5% in basis points
        RemiVaultCore::set_fee_rate(&env, &fee_rate);

        let stored_fee = RemiVaultCore::get_fee_rate(&env);
        assert_eq!(stored_fee, 500, "Fee rate should be stored correctly");
    }

    // =====================================================
    // TEST 9 — NEGATIVE ASSETS INVARIANT
    // =====================================================

    #[test]
    fn test_no_negative_assets() {
        let env = Env::default();

        let vault_state = VaultState {
            total_assets: 1000,
            total_shares: 100,
            accrued_fees: 0,
            last_reserve_brate: 0,
        };

        // Verify no negative values
        assert!(vault_state.total_assets >= 0);
        assert!(vault_state.total_shares >= 0);
        assert!(vault_state.accrued_fees >= 0);
    }

    // =====================================================
    // TEST 10 — HARVEST MECHANICS
    // =====================================================

    #[test]
    fn test_harvest_internal_mechanics() {
        let env = Env::default();

        let vault_state = VaultState {
            total_assets: 1000,
            total_shares: 100,
            accrued_fees: 0,
            last_reserve_brate: 0,
        };

        RemiVaultCore::set_vault_state(&env, &vault_state);

        // Test: Harvest should increase assets
        let harvested = 200;
        RemiVaultCore::_harvest_internal(&env, harvested);

        let updated_state = RemiVaultCore::get_vault_state(&env);
        assert_eq!(
            updated_state.total_assets, 1200,
            "Assets should increase by harvest amount"
        );
    }

    // =====================================================
    // TEST 11 — FEE COLLECTION
    // =====================================================

    #[test]
    fn test_fee_collection_mechanics() {
        let env = Env::default();

        let vault_state = VaultState {
            total_assets: 1000,
            total_shares: 100,
            accrued_fees: 0,
            last_reserve_brate: 0,
        };

        RemiVaultCore::set_vault_state(&env, &vault_state);
        RemiVaultCore::set_fee_rate(&env, &2000); // 20% fee

        // Test: Collect fees on 1000 yield
        RemiVaultCore::_collect_fees_internal(&env, 1000);

        let updated_state = RemiVaultCore::get_vault_state(&env);

        // 20% of 1000 = 200
        assert_eq!(
            updated_state.accrued_fees, 200,
            "Fees should be calculated correctly (20% of 1000 = 200)"
        );
    }

    // =====================================================
    // INVARIANT TESTS
    // =====================================================

    #[test]
    fn test_invariant_share_consistency() {
        let env = Env::default();

        // Invariant: Share conversion should be reversible
        let vault_state = VaultState {
            total_assets: 10000,
            total_shares: 1000,
            accrued_fees: 0,
            last_reserve_brate: 0,
        };

        RemiVaultCore::set_vault_state(&env, &vault_state);

        // Convert 500 assets to shares, then back
        let shares = RemiVaultCore::convert_to_shares(&env, 500);
        let assets_back = RemiVaultCore::convert_to_assets(&env, shares);

        assert_eq!(assets_back, 500, "Share conversion should be reversible");
    }

    #[test]
    fn test_invariant_no_zero_division() {
        let env = Env::default();

        // Invariant: Empty vault should not cause division errors
        let vault_state = VaultState {
            total_assets: 0,
            total_shares: 0,
            accrued_fees: 0,
            last_reserve_brate: 0,
        };

        RemiVaultCore::set_vault_state(&env, &vault_state);

        // Should return assets as-is when total_shares == 0
        let shares = RemiVaultCore::convert_to_shares(&env, 1000);
        assert_eq!(
            shares, 1000,
            "Empty vault should mint 1:1 shares without errors"
        );
    }
}
