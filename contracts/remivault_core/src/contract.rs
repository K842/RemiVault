use soroban_sdk::{
    contract,
    contractimpl,
    contracterror,
    panic_with_error,
    token,
    Address,
    Env,
};

use crate::blend::BlendClient;
use crate::storage::{
    DataKey,
    Config,
    VaultState,
    PauseLevel,
};
use crate::events;

// =====================================================
// ERROR ENUM
// =====================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    InsufficientShares = 1,
    InsufficientBalance = 2,
    VaultPaused = 3,
    AllowanceExceeded = 4,
    InvalidAmount = 5,
    Unauthorized = 6,
}

// =====================================================
// CONTRACT
// =====================================================

#[contract]
pub struct RemiVaultCore;

#[contractimpl]
impl RemiVaultCore {

    // =====================================================
    // INITIALIZATION
    // =====================================================

    pub fn initialize(
        e: Env,
        asset: Address,
        admin: Address,
        treasury: Address,
        blend_pool: Address,
        fee_rate_bps: i128,
    ) {

        // Prevent re-initialization
        if e.storage().instance().has(&DataKey::Config) {
            panic_with_error!(
                &e,
                ContractError::Unauthorized
            );
        }

        // Create config
        let config = Config {
            asset,
            admin,
            treasury,
            blend_pool,
        };

        // Save config
        Self::set_config(&e, config);

        // Create initial vault state
        let vault_state = VaultState {
            total_shares: 0,
            total_assets: 0,
            accrued_fees: 0,
            last_reserve_brate: 0,
        };

        // Save vault state
        Self::set_vault_state(&e, vault_state);

        // Set pause level
        Self::set_pause_level(
            &e,
            PauseLevel::None,
        );

        // Set fee rate
        Self::set_fee_rate(
            &e,
            fee_rate_bps,
        );
    }

    // =====================================================
    // CONFIG STORAGE
    // =====================================================

    pub fn get_config(e: &Env) -> Config {
        e.storage()
            .instance()
            .get(&DataKey::Config)
            .unwrap()
    }

    pub fn set_config(
        e: &Env,
        config: Config,
    ) {
        e.storage()
            .instance()
            .set(&DataKey::Config, &config);
    }

    // =====================================================
    // VAULT STATE STORAGE
    // =====================================================

    pub fn get_vault_state(
        e: &Env,
    ) -> VaultState {
        e.storage()
            .persistent()
            .get(&DataKey::VaultState)
            .unwrap()
    }

    pub fn set_vault_state(
        e: &Env,
        state: VaultState,
    ) {
        e.storage()
            .persistent()
            .set(&DataKey::VaultState, &state);
    }

    // =====================================================
    // USER SHARE STORAGE
    // =====================================================

    pub fn get_user_shares(
        e: &Env,
        user: Address,
    ) -> i128 {
        e.storage()
            .persistent()
            .get(&DataKey::UserShares(user))
            .unwrap_or(0)
    }

    pub fn set_user_shares(
        e: &Env,
        user: Address,
        shares: i128,
    ) {
        e.storage()
            .persistent()
            .set(
                &DataKey::UserShares(user),
                &shares,
            );
    }

    // =====================================================
    // PAUSE LEVEL STORAGE
    // =====================================================

    pub fn get_pause_level(
        e: &Env,
    ) -> PauseLevel {
        e.storage()
            .instance()
            .get(&DataKey::PauseLevel)
            .unwrap_or(PauseLevel::None)
    }

    pub fn set_pause_level(
        e: &Env,
        level: PauseLevel,
    ) {
        e.storage()
            .instance()
            .set(
                &DataKey::PauseLevel,
                &level,
            );
    }

    // =====================================================
    // ADMIN SECURITY
    // =====================================================

    fn require_admin(
        e: &Env,
    ) {
        let config =
            Self::get_config(e);

        config.admin.require_auth();
    }

    // =====================================================
    // FEE RATE STORAGE
    // =====================================================

    pub fn get_fee_rate(
        e: &Env,
    ) -> i128 {
        e.storage()
            .instance()
            .get(&DataKey::FeeRateBps)
            .unwrap()
    }

    pub fn set_fee_rate(
        e: &Env,
        fee_rate: i128,
    ) {
        e.storage()
            .instance()
            .set(
                &DataKey::FeeRateBps,
                &fee_rate,
            );
    }

    // =====================================================
    // VAULT MATH
    // =====================================================

    pub fn convert_to_shares(
        e: &Env,
        assets: i128,
    ) -> i128 {

        let state = Self::get_vault_state(e);

        if state.total_shares == 0 {
            assets
        } else {
            // Use checked multiplication and division with overflow protection
            assets.checked_mul(state.total_shares)
                .expect("convert_to_shares: overflow in multiplication")
                / state.total_assets
        }
    }

    pub fn convert_to_assets(
        e: &Env,
        shares: i128,
    ) -> i128 {

        let state = Self::get_vault_state(e);

        if state.total_shares == 0 {
            shares
        } else {
            // Use checked multiplication with overflow protection
            shares.checked_mul(state.total_assets)
                .expect("convert_to_assets: overflow in multiplication")
                / state.total_shares
        }
    }

    // =====================================================
    // DEPOSIT
    // =====================================================

    pub fn deposit(
        e: Env,
        user: Address,
        assets: i128,
    ) {

        // Validate amount
        if assets <= 0 {
            panic_with_error!(
                &e,
                ContractError::InvalidAmount
            );
        }

        // Auth check
        user.require_auth();

        // Pause check
        let pause_level =
            Self::get_pause_level(&e);

        if pause_level
            == PauseLevel::EmergencyPause
        {
            panic_with_error!(
                &e,
                ContractError::VaultPaused
            );
        }

        // Load config
        let config =
            Self::get_config(&e);

        // Create token client
        let token_client =
            token::Client::new(
                &e,
                &config.asset,
            );

        // Transfer assets into vault
        token_client.transfer(
            &user,
            &e.current_contract_address(),
            &assets,
        );

        // Calculate shares
        let shares =
            Self::convert_to_shares(
                &e,
                assets,
            );

        // Load user balance
        let user_shares =
            Self::get_user_shares(
                &e,
                user.clone(),
            );

        // Update user shares with overflow protection
        Self::set_user_shares(
            &e,
            user.clone(),
            user_shares.checked_add(shares)
                .expect("Deposit: user shares overflow"),
        );

        // Update vault state with overflow protection
        let mut state =
            Self::get_vault_state(&e);

        state.total_assets = state.total_assets
            .checked_add(assets)
            .expect("Deposit: total_assets overflow");
        state.total_shares = state.total_shares
            .checked_add(shares)
            .expect("Deposit: total_shares overflow");

        // Save updated state
        Self::set_vault_state(
            &e,
            state,
        );
        Self::supply_to_blend(
    &e,
    assets,
);

        // Emit deposit event using event module
        events::emit_deposit(&e, user, assets, shares);
    }

    // =====================================================
    // WITHDRAW
    // =====================================================

    pub fn withdraw(
        e: Env,
        user: Address,
        shares: i128,
    ) {

        // Validate amount
        if shares <= 0 {
            panic_with_error!(
                &e,
                ContractError::InvalidAmount
            );
        }

        // Auth check
        user.require_auth();

        // Pause check
        let pause_level =
            Self::get_pause_level(&e);

        if pause_level
            == PauseLevel::EmergencyPause
        {
            panic_with_error!(
                &e,
                ContractError::VaultPaused
            );
        }

        // Load user shares
        let user_shares =
            Self::get_user_shares(
                &e,
                user.clone(),
            );

        // Balance validation
        if user_shares < shares {
            panic_with_error!(
                &e,
                ContractError::InsufficientShares
            );
        }

        // Convert shares -> assets
        let assets =
            Self::convert_to_assets(
                &e,
                shares,
            );

        // Burn shares
        Self::set_user_shares(
            &e,
            user.clone(),
            user_shares.checked_sub(shares)
                .expect("Withdraw: user shares underflow"),
        );

        // Update vault state with overflow protection
        let mut state =
            Self::get_vault_state(&e);

        state.total_assets = state.total_assets
            .checked_sub(assets)
            .expect("Withdraw: total_assets underflow");
        state.total_shares = state.total_shares
            .checked_sub(shares)
            .expect("Withdraw: total_shares underflow");

        // Save updated state
        Self::set_vault_state(
            &e,
            state,
        );

        // Load config
        let config =
            Self::get_config(&e);

        // Create token client
        let token_client =
            token::Client::new(
                &e,
                &config.asset,
            );
            Self::withdraw_from_blend(
                &e,
                assets,
                );

        // Transfer assets back
        token_client.transfer(
            &e.current_contract_address(),
            &user,
            &assets,
        );

        // Emit withdraw event using event module
        events::emit_withdraw(&e, user, assets, shares);
    }

    // =====================================================
    // READ METHODS
    // =====================================================

    pub fn get_user_balance(
        e: Env,
        user: Address,
    ) -> i128 {
        Self::get_user_shares(
            &e,
            user,
        )
    }

    pub fn vault_state(
        e: Env,
    ) -> VaultState {
        Self::get_vault_state(&e)
    }


// =====================================================
// SEP-41 Share Token Layer
// │
// ├── balance()
// ├── allowance()
// ├── approve()
// ├── transfer()
// ├── transfer_from()
// ├── decimals()
// ├── name()
// └── symbol()
// =====================================================
pub fn balance(
    e:Env,
    user:Address,
) -> i128 {
    Self::get_user_shares(
        &e,
        user,
    )
}
pub fn transfer(e:Env,from:Address,to:Address,amount:i128){
    // Validate amount
    if amount <= 0 {
        panic_with_error!(
            &e,
            ContractError::InvalidAmount
        );
    }
    
    //auth check 
    from.require_auth();
    //balance check
    let from_balance=Self::get_user_shares(
        &e,
        from.clone(),
    );
    //Validate
    if from_balance < amount{
        panic_with_error!(
            &e,
            ContractError::InsufficientBalance
        ); 
    }
    //load the receiver balance
    let to_balance= Self::get_user_shares(&e,to.clone());
    //update sender balance with underflow protection
    Self::set_user_shares(&e, from.clone(), from_balance
        .checked_sub(amount)
        .expect("Transfer: from balance underflow"));
    //update receiver balance with overflow protection
    Self::set_user_shares(&e,to.clone(),to_balance
        .checked_add(amount)
        .expect("Transfer: to balance overflow"));
    
    // Emit transfer event using event module
    e.events().publish(
        ("transfer", from.clone(), to),
        amount,
    );
 }
 pub fn allowance(e: Env, owner: Address, spender: Address) -> i128 {
    e.storage()
        .persistent()
        .get(
            &DataKey::Allowance(owner, spender)
        )
        .unwrap_or(0)
 }
 pub fn approve(e: Env, owner:Address,spender:Address,amount:i128){
    owner.require_auth(); 
    e.storage()
    .persistent()
    .set(
        &DataKey::Allowance(owner.clone(),spender.clone()),&amount
    );
    
    // Emit approve event
    e.events().publish(
        ("approve", owner, spender),
        amount,
    );   
 }
 pub fn transfer_from(
    e: Env,
    spender: Address,
    from: Address,
    to: Address,
    amount: i128,
) {

    // Auth check
    spender.require_auth();

    // Load allowance
    let allowance =
        Self::allowance(
            e.clone(),
            from.clone(),
            spender.clone(),
        );

    // Validate allowance
    if allowance < amount {
        panic_with_error!(
            &e,
            ContractError::AllowanceExceeded
        );
    }

    // Load owner balance
    let from_balance =
        Self::get_user_shares(
            &e,
            from.clone(),
        );

    // Validate owner balance
    if from_balance < amount {
        panic_with_error!(
            &e,
            ContractError::InsufficientBalance
        );
    }

    // Reduce allowance with underflow protection
    e.storage()
        .persistent()
        .set(
            &DataKey::Allowance(
                from.clone(),
                spender,
            ),
            &(allowance.checked_sub(amount)
                .expect("Transfer_from: allowance underflow")),
        );

    // Load receiver balance
    let to_balance =
        Self::get_user_shares(
            &e,
            to.clone(),
        );

    // Update owner balance with underflow protection
    Self::set_user_shares(
        &e,
        from.clone(),
        from_balance.checked_sub(amount)
            .expect("Transfer_from: from balance underflow"),
    );

    // Update receiver balance with overflow protection
    Self::set_user_shares(
        &e,
        to.clone(),
        to_balance.checked_add(amount)
            .expect("Transfer_from: to balance overflow"),
    );

    // Emit transfer event
    e.events().publish(
        ("transfer", from, to),
        amount,
    );
}
// =====================================================
// TOKEN METADATA
// =====================================================

pub fn name(
    e: Env,
) -> soroban_sdk::String {

    soroban_sdk::String::from_str(
        &e,
        "Remi Vault Share",
    )
}

pub fn symbol(
    e: Env,
) -> soroban_sdk::String {

    soroban_sdk::String::from_str(
        &e,
        "rUSDC",
    )
}

pub fn decimals() -> u32 {
    6
}

// =====================================================
// ADMIN OPERATIONS
// =====================================================

pub fn harvest(e: Env) {
    // Admin check
    RemiVaultCore::require_admin(&e);

    // Sync yield with Blend
    RemiVaultCore::sync_yield(&e);
}

pub fn update_pause_level(
    e: Env,
    level: PauseLevel,
) {
    // Admin check
    Self::require_admin(&e);

    // Store pause level
    e.storage()
        .instance()
        .set(
            &DataKey::PauseLevel,
            &level,
        );

    // Emit pause level updated event
    e.events().publish(
        ("pause_level_updated",),
        level,
    );
}
fn _harvest_internal(
    e: &Env,
    harvested_assets: i128,
) {
    // Admin check
    Self::require_admin(e);

    // Load vault state
    let mut state =
        Self::get_vault_state(e);

    // Increase vault assets with overflow protection
    state.total_assets = state.total_assets
        .checked_add(harvested_assets)
        .expect("Harvest: total_assets overflow");

    // Save updated state
    Self::set_vault_state(
        e,
        state,
    );

    // Emit harvest event using event module
    events::emit_harvest(e, harvested_assets);
}

fn _collect_fees_internal(
    e: &Env,
    yield_amount: i128,
) {
    // Admin check
    Self::require_admin(e);

    // Load vault state
    let mut state =
        Self::get_vault_state(e);

    // Load fee rate
    let fee_rate =
        Self::get_fee_rate(e);

    // Calculate fee (in basis points) with overflow protection
    let fee_amount =
        (yield_amount.checked_mul(fee_rate)
            .expect("Fee calculation: overflow")) / 10000;

    // Accrue fees to treasury with overflow protection
    state.accrued_fees = state.accrued_fees
        .checked_add(fee_amount)
        .expect("Fee collection: accrued_fees overflow");

    // Save updated state
    Self::set_vault_state(
        e,
        state,
    );

    // Emit fees collected event using event module
    events::emit_fee_collection(e, fee_amount);
}




fn supply_to_blend(
    e: &Env,
    amount: i128,
) {

    // Load config
    let config =
        Self::get_config(e);

    // Create Blend client
    let blend_client =
        BlendClient::new(
            e,
            &config.blend_pool,
        );

    // Supply assets into Blend
    blend_client.supply(
        &e.current_contract_address(),
        &config.asset,
        amount,
    );

    // Update reserve tracking with overflow protection
    let mut state =
        Self::get_vault_state(e);

    state.last_reserve_brate = state.last_reserve_brate
        .checked_add(amount)
        .expect("Supply to Blend: last_reserve_brate overflow");

    // Save updated state
    Self::set_vault_state(
        e,
        state,
    );
}


fn sync_yield(
    e: &Env,
) {
    // Load vault state
    let mut state =
        Self::get_vault_state(e);

    // Load config for Blend integration
    let config = Self::get_config(e);

    // Create Blend client
    let blend_client =
        BlendClient::new(
            e,
            &config.blend_pool,
        );

    // Query current position from Blend
    let current_position =
        blend_client.get_position(
            &e.current_contract_address(),
        );

    // Previous reserve baseline (stored in state)
    let previous_position =
        state.last_reserve_brate;

    // Calculate earned yield (from interest accrual)
    let earned_yield =
        current_position.supplied
        .checked_sub(previous_position)
        .unwrap_or(0);

    // Harvest profits if positive
    if earned_yield > 0 {
        Self::_harvest_internal(
            e,
            earned_yield,
        );

        Self::_collect_fees_internal(
            e,
            earned_yield,
        );

        // Update state with new baseline
        state.last_reserve_brate = current_position.supplied;
    }

    // Save updated state
    Self::set_vault_state(
        e,
        state,
    );
}




fn withdraw_from_blend(
    e: &Env,
    amount: i128,
) {

    // Load config
    let config =
        Self::get_config(e);

    // Create Blend client
    let blend_client =
        BlendClient::new(
            e,
            &config.blend_pool,
        );

    // Withdraw assets from Blend
    blend_client.withdraw(
        &e.current_contract_address(),
        &config.asset,
        amount,
    );

    // Update reserve tracking with underflow protection
    let mut state =
        Self::get_vault_state(e);

    state.last_reserve_brate = state.last_reserve_brate
        .checked_sub(amount)
        .expect("Withdraw from Blend: last_reserve_brate underflow");

    // Save updated state
    Self::set_vault_state(
        e,
        state,
    );
}
}