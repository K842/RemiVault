use soroban_sdk::{
    contract,
    contractimpl,
    contracttype,
    contracterror,
    panic_with_error,
    token,
    Address,
    Env,
};

use crate::blend::BlendClient;

// =====================================================
// STORAGE TYPES
// =====================================================

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Config,
    VaultState,
    PauseLevel,
    FeeRateBps,
    UserShares(Address),
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
#[derive(Clone, PartialEq)]
pub enum PauseLevel {
    None,
    DepositOnly,
    EmergencyPause,
}

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
        Self::set_config(&e, &config);

        // Create initial vault state
        let vault_state = VaultState {
            total_shares: 0,
            total_assets: 0,
            accrued_fees: 0,
            last_reserve_brate: 0,
        };

        // Save vault state
        Self::set_vault_state(&e, &vault_state);

        // Set pause level
        Self::set_pause_level(
            &e,
            &PauseLevel::None,
        );

        // Set fee rate
        Self::set_fee_rate(
            &e,
            &fee_rate_bps,
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
        config: &Config,
    ) {
        e.storage()
            .instance()
            .set(&DataKey::Config, config);
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
        state: &VaultState,
    ) {
        e.storage()
            .persistent()
            .set(&DataKey::VaultState, state);
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
        level: &PauseLevel,
    ) {
        e.storage()
            .instance()
            .set(
                &DataKey::PauseLevel,
                level,
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
            // Note: Production systems should use checked_add/checked_mul
            // to prevent overflow. Current i128 capacity is ~9.2e18.
            (assets * state.total_shares)
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
            // Note: Production systems should use checked_add/checked_mul
            // to prevent overflow. Current i128 capacity is ~9.2e18.
            (shares * state.total_assets)
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

        // Update user shares
        Self::set_user_shares(
            &e,
            user,
            user_shares + shares,
        );

        // Update vault state
        let mut state =
            Self::get_vault_state(&e);

        state.total_assets += assets;
        state.total_shares += shares;

        // Save updated state
        Self::set_vault_state(
            &e,
            &state,
        );
        Self::supply_to_blend(
    &e,
    assets,
);

        // Emit deposit event
        e.events().publish(
            ("deposit", user),
            assets,
        );
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
            user_shares - shares,
        );

        // Update vault state
        let mut state =
            Self::get_vault_state(&e);

        state.total_assets -= assets;
        state.total_shares -= shares;

        // Save updated state
        Self::set_vault_state(
            &e,
            &state,
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

        // Emit withdraw event
        e.events().publish(
            ("withdraw", user),
            assets,
        );
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
    //update sender balance
    Self::set_user_shares(&e, from.clone(), from_balance-amount);
    //update receiver balance
    Self::set_user_shares(&e,to.clone(),to_balance+amount);
    
    // Emit transfer event
    e.events().publish(
        ("transfer", from, to),
        amount,
    );
 }
 pub fn allowance(e:Evn,owner: Address,Spender:Address,)-> i128{
    e.storage()
        .persistent(),
        .get(
            &DataKey::Allowance(owner,spender,)
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

    // Reduce allowance
    e.storage()
        .persistent()
        .set(
            &DataKey::Allowance(
                from.clone(),
                spender,
            ),
            &(allowance - amount),
        );

    // Load receiver balance
    let to_balance =
        Self::get_user_shares(
            &e,
            to.clone(),
        );

    // Update owner balance
    Self::set_user_shares(
        &e,
        from,
        from_balance - amount,
    );

    // Update receiver balance
    Self::set_user_shares(
        &e,
        to.clone(),
        to_balance + amount,
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
) -> String {

    String::from_str(
        &e,
        "Remi Vault Share",
    )
}

pub fn symbol(
    e: Env,
) -> String {

    String::from_str(
        &e,
        "rUSDC",
    )
}

pub fn decimals() -> u32 {
    7
}

// =====================================================
// ADMIN OPERATIONS
// =====================================================

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

    // Load vault state
    let mut state =
        Self::get_vault_state(e);

    // Increase vault assets
    // Note: In production, use checked_add to prevent overflow
    state.total_assets += harvested_assets;

    // Save updated state
    Self::set_vault_state(
        e,
        &state,
    );

    // Emit harvest event
    e.events().publish(
        ("harvest",),
        harvested_assets,
    );
}

fn _collect_fees_internal(
    e: &Env,
    yield_amount: i128,
) {

    // Load vault state
    let mut state =
        Self::get_vault_state(e);

    // Load fee rate
    let fee_rate =
        Self::get_fee_rate(e);

    // Calculate fee (in basis points)
    let fee_amount =
        (yield_amount * fee_rate) / 10000;

    // Accrue fees to treasury
    // Note: In production, use checked_add to prevent overflow
    state.accrued_fees += fee_amount;

    // Save updated state
    Self::set_vault_state(
        e,
        &state,
    );

    // Emit fees collected event
    e.events().publish(
        ("fees_collected",),
        fee_amount,
    );
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
        &amount,
    );

    // Update reserve tracking
    let mut state =
        Self::get_vault_state(e);

    state.last_reserve_brate += amount;

    // Save updated state
    Self::set_vault_state(
        e,
        &state,
    );
}

fn sync_yield(
    e: &Env,
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

    // Query current reserve position
    let current_position =
        blend_client.get_position(
            &e.current_contract_address(),
            &config.asset,
        );

    // Load vault state
    let mut state =
        Self::get_vault_state(e);

    // Previous reserve baseline
    let previous_position =
        state.last_reserve_brate;

    // Calculate earned yield
    let earned_yield =
        current_position
        - previous_position;

    // Harvest profits
    if earned_yield > 0 {

        Self::_harvest_internal(
            e,
            earned_yield,
        );

        Self::_collect_fees_internal(
            e,
            earned_yield,
        );
    }

    // Update reserve baseline
    state.last_reserve_brate =
        current_position;

    // Save updated state
    Self::set_vault_state(
        e,
        &state,
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
        &amount,
    );

    // Update reserve tracking
    let mut state =
        Self::get_vault_state(e);

    state.last_reserve_brate -= amount;

    // Save updated state
    Self::set_vault_state(
        e,
        &state,
    );
}

}
}