use solana_sdk::pubkey::Pubkey;

pub const TOKEN_PROGRAM: Pubkey = solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const TOKEN_2022_PROGRAM: Pubkey = solana_sdk::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
pub const WSOL_MINT: Pubkey = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");
pub const ASSOCIATED_TOKEN_PROGRAM: Pubkey = solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenProgram {
    Legacy,
    Token2022,
}

impl TokenProgram {
    #[must_use]
    pub fn id(self) -> Pubkey {
        match self {
            Self::Legacy => TOKEN_PROGRAM,
            Self::Token2022 => TOKEN_2022_PROGRAM,
        }
    }
}
