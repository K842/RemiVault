use soroban_sdk::{
    contractclient,
    contracttype,
    Address,
    Env,
    Vec,
};

// =====================================================
// BLEND POSITION STRUCTURE
// Represents vault reserve ownership inside Blend
// =====================================================

#[contracttype]
#[derive(Clone)]
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

#[contracttype]
#[derive(Clone)]
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

#[contracttype]
#[derive(Clone)]
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

// =====================================================
// BLEND CLIENT WRAPPER
// Simplified interface for vault operations
// =====================================================

#[allow(dead_code)]
pub struct BlendClient<'env> {
    env: &'env Env,
    blend_pool: &'env Address,
}

impl<'env> BlendClient<'env> {
    pub fn new(env: &'env Env, blend_pool: &'env Address) -> Self {
        BlendClient { env, blend_pool }
    }

    // Supply wrapper: creates and submits a supply request to Blend
    pub fn supply(
        &self,
        from: &Address,
        asset: &Address,
        amount: i128,
    ) {
        // Create supply request (request_type = 0)
        let request = Request {
            request_type: 0,
            asset: asset.clone(),
            amount,
        };

        // Build request vector
        let mut requests = Vec::new(self.env);
        requests.push_back(request);

        // Create Blend pool client
        let blend_pool_client =
            BlendPoolClient::new(self.env, self.blend_pool);

        // Submit supply request to Blend
        blend_pool_client.submit(
            from,
            from,
            from,
            &requests,
        );
    }

    // Withdraw wrapper: creates and submits a withdraw request to Blend
    pub fn withdraw(
        &self,
        from: &Address,
        asset: &Address,
        amount: i128,
    ) {
        // Create withdraw request (request_type = 1)
        let request = Request {
            request_type: 1,
            asset: asset.clone(),
            amount,
        };

        // Build request vector
        let mut requests = Vec::new(self.env);
        requests.push_back(request);

        // Create Blend pool client
        let blend_pool_client =
            BlendPoolClient::new(self.env, self.blend_pool);

        // Submit withdraw request to Blend
        blend_pool_client.submit(
            from,
            from,
            from,
            &requests,
        );
    }

    // Query current position in Blend
    pub fn get_position(
        &self,
        user: &Address,
    ) -> PoolPosition {
        let blend_pool_client =
            BlendPoolClient::new(self.env, self.blend_pool);

        blend_pool_client.get_position(user)
    }

    // Query current lending rates
    pub fn get_rates(
        &self,
        asset: &Address,
    ) -> RateData {
        let blend_pool_client =
            BlendPoolClient::new(self.env, self.blend_pool);

        blend_pool_client.get_rates(asset)
    }
}