use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Copy)]
pub enum AuthoritySource {
    Payer,
    Slot(&'static str),
    Fixed(Pubkey),
}

#[derive(Debug, Clone, Copy)]
pub struct NonceConfig {
    pub account_slot: &'static str,
    pub authority: AuthoritySource,
}

#[derive(Debug, Clone, Copy)]
pub struct ComputeBudgetSlots {
    pub limit_slot: &'static str,
    pub price_slot: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct TipTransferSlots {
    pub account_slot: &'static str,
    pub lamports_slot: &'static str,
    pub per_provider: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PrefixOptions {
    pub advance_nonce: Option<NonceConfig>,
    pub compute_budget: Option<ComputeBudgetSlots>,
    pub tip_transfer: Option<TipTransferSlots>,
}

impl PrefixOptions {
    #[must_use]
    pub fn with_advance_nonce(mut self, cfg: NonceConfig) -> Self {
        self.advance_nonce = Some(cfg);
        self
    }

    #[must_use]
    pub fn with_compute_budget(mut self, cfg: ComputeBudgetSlots) -> Self {
        self.compute_budget = Some(cfg);
        self
    }

    #[must_use]
    pub fn with_tip_transfer(mut self, cfg: TipTransferSlots) -> Self {
        self.tip_transfer = Some(cfg);
        self
    }
}
