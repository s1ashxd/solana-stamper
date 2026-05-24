use std::panic::AssertUnwindSafe;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use smallvec::smallvec;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;

use tx_stamper::protocols::common::{TOKEN_PROGRAM, WSOL_MINT, TokenProgram};
use tx_stamper::protocols::damm_v2::constants::DAMM_V2_PROGRAM;
use tx_stamper::protocols::damm_v2::pda::pool_authority as damm_pool_authority;
use tx_stamper::protocols::printr::constants::{DBC_PROGRAM, PRINTR_PROGRAM};
use tx_stamper::protocols::pumpfun::constants::PUMPFUN_PROGRAM;
use tx_stamper::protocols::pumpswap::constants::PUMPSWAP_PROGRAM;
use tx_stamper::signer::{KeypairSigner, Signer};
use tx_stamper::stamp::bundle::PerProviderValues;
use tx_stamper::template::Template;

const RPC_URL: &str = "https://api.mainnet-beta.solana.com";

const PUMPFUN_MINT: Pubkey =
    solana_sdk::pubkey!("6qP8N3GvJRL3dK9ki7x9zfjqw8WVg5Hjc8YLXSNzpump");
const PUMPSWAP_MINT: Pubkey =
    solana_sdk::pubkey!("7gi5VSix48urNKD1NifBYjTAn5e7WaSKKuDp3Tm6pump");
const DAMM_V2_MINT: Pubkey =
    solana_sdk::pubkey!("6VTv46gGe3e5jxEpCfwsNZUDMtJw7ssQif5PntXhHyAV");
const PRINTR_MINT: Pubkey =
    solana_sdk::pubkey!("HRTtbAKuxcHRkYvRmBEpXYjf3s9d1FNocHCpnnW8brrr");

const JITO_TIP: Pubkey = solana_sdk::pubkey!("96gYZGLRJq7Tq3wcKqSLwhKf3xn5GHHGzVDqFAjksFnY");
const PRINTR_PARTNER_CONFIG: Pubkey =
    solana_sdk::pubkey!("A8gMrEPJkacWkcb3DGwtJwTe16HktSEfvwtuDh2MCtck");
const PRINTR_LUT_ADDR: Pubkey = solana_sdk::pubkey!("7RKtfATWCe98ChuwecNq8XCzAzfoK3DtZTprFsPMGtio");
const PUMPSWAP_FEE_RECIPIENT: Pubkey =
    solana_sdk::pubkey!("62qc2CNXwrYqQScmEdiZFFAnJR262PxWEuNQtxfafNgV");

fn load_keypair_from_env() -> KeypairSigner {
    let Ok(path) = std::env::var("SOLANA_KEYPAIR") else {
        eprintln!("SOLANA_KEYPAIR env var not set");
        std::process::exit(1);
    };
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("failed to read keypair file {path}: {e}");
        std::process::exit(1);
    });
    let trimmed = raw.trim().trim_start_matches('[').trim_end_matches(']');
    let bytes: Vec<u8> = trimmed
        .split(',')
        .map(|s| {
            s.trim().parse::<u8>().unwrap_or_else(|e| {
                eprintln!("failed to parse keypair byte: {e}");
                std::process::exit(1);
            })
        })
        .collect();
    if bytes.len() != 64 {
        eprintln!("keypair file must contain 64 bytes, got {}", bytes.len());
        std::process::exit(1);
    }
    let secret: [u8; 32] = bytes[..32].try_into().expect("slice length checked");
    KeypairSigner::from_bytes(&secret)
}

fn ata(owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address_with_program_id(
        owner,
        mint,
        token_program,
    )
}

fn pda(seeds: &[&[u8]], program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(seeds, program).0
}

fn print_sim_result(
    protocol: &str,
    result: &solana_client::client_error::Result<
        solana_client::rpc_response::Response<
            solana_client::rpc_response::RpcSimulateTransactionResult,
        >,
    >,
) {
    println!("=== {protocol} ===");
    match result {
        Err(e) => println!("RPC error: {e}"),
        Ok(resp) => {
            let val = &resp.value;
            println!("err: {:?}", val.err);
            println!("units_consumed: {:?}", val.units_consumed);
            if let Some(logs) = &val.logs {
                let tail: Vec<&String> = logs.iter().rev().take(10).collect::<Vec<_>>().into_iter().rev().collect();
                println!("last {} log lines:", tail.len());
                for line in tail {
                    println!("  {line}");
                }
            } else {
                println!("logs: none");
            }
        }
    }
    println!();
}

fn sim_config() -> RpcSimulateTransactionConfig {
    RpcSimulateTransactionConfig {
        sig_verify: true,
        replace_recent_blockhash: false,
        commitment: Some(CommitmentConfig::confirmed()),
        encoding: None,
        accounts: None,
        min_context_slot: None,
        inner_instructions: false,
    }
}

fn simulate_pumpfun(rpc: &RpcClient, keypair: &KeypairSigner) {
    let payer = keypair.pubkey();
    let mint = PUMPFUN_MINT;

    let bonding_curve = pda(&[b"bonding-curve", mint.as_ref()], &PUMPFUN_PROGRAM);
    let associated_bonding_curve =
        ata(&bonding_curve, &mint, &TOKEN_PROGRAM);
    let bonding_curve_v2 = pda(&[b"bonding-curve-v2", mint.as_ref()], &PUMPFUN_PROGRAM);
    let user_vol = pda(
        &[b"user_volume_accumulator", payer.as_ref()],
        &PUMPFUN_PROGRAM,
    );

    let bc_data = match rpc.get_account_data(&bonding_curve) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("pumpfun: failed to fetch bonding_curve account: {e}");
            return;
        }
    };
    if bc_data.len() < 81 {
        eprintln!("pumpfun: bonding_curve account data too short ({})", bc_data.len());
        return;
    }
    let creator = Pubkey::new_from_array(bc_data[49..81].try_into().unwrap());
    let creator_vault = pda(&[b"creator-vault", creator.as_ref()], &PUMPFUN_PROGRAM);

    println!("pumpfun slots:");
    println!("  mint: {mint}");
    println!("  bonding_curve: {bonding_curve}");
    println!("  associated_bonding_curve: {associated_bonding_curve}");
    println!("  creator: {creator}");
    println!("  creator_vault: {creator_vault}");
    println!("  bonding_curve_v2: {bonding_curve_v2}");
    println!("  user_vol: {user_vol}");

    let spec = tx_stamper::protocols::pumpfun::buy_v2_spec(payer, TokenProgram::Legacy);
    let tpl = match Template::compile(spec) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("pumpfun: compile error: {e}");
            return;
        }
    };

    let blockhash = match rpc.get_latest_blockhash() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("pumpfun: get_latest_blockhash error: {e}");
            return;
        }
    };

    let provider = PerProviderValues {
        slots: smallvec![
            ("tip_account".to_string(), JITO_TIP.into()),
            ("tip_lamports".to_string(), 1_000u64.into()),
        ],
    };

    let bundle = match tpl
        .stamp_bundle([provider])
        .set("mint", mint)
        .set("bonding_curve", bonding_curve)
        .set("associated_bonding_curve", associated_bonding_curve)
        .set("creator_vault", creator_vault)
        .set("bonding_curve_v2", bonding_curve_v2)
        .set("user_vol", user_vol)
        .set("sol_amount", 10_000u64)
        .set("min_tokens_out", 1u64)
        .set("cu_limit", 200_000u32)
        .set("cu_price", 100_000u64)
        .blockhash(blockhash)
        .sign(keypair)
    {
        Ok(b) => b,
        Err(e) => {
            eprintln!("pumpfun: stamp error: {e}");
            return;
        }
    };

    let stamped = bundle.reconstruct(0);
    let tx: VersionedTransaction = match bincode::deserialize(stamped.as_bytes()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("pumpfun: deserialize error: {e}");
            return;
        }
    };

    let result = rpc.simulate_transaction_with_config(&tx, sim_config());
    print_sim_result("PumpFun", &result);
}

fn simulate_pumpswap(rpc: &RpcClient, keypair: &KeypairSigner) {
    let payer = keypair.pubkey();
    let base_mint = PUMPSWAP_MINT;

    let pump_pool_authority = pda(
        &[b"pool-authority", base_mint.as_ref()],
        &PUMPFUN_PROGRAM,
    );
    let index_le = 0u16.to_le_bytes();
    let pool = pda(
        &[
            b"pool",
            &index_le,
            pump_pool_authority.as_ref(),
            base_mint.as_ref(),
            WSOL_MINT.as_ref(),
        ],
        &PUMPSWAP_PROGRAM,
    );

    let pool_data = match rpc.get_account_data(&pool) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("pumpswap: failed to fetch pool account: {e}");
            return;
        }
    };
    if pool_data.len() < 243 {
        eprintln!(
            "pumpswap: pool account data too short ({} bytes); pool not migrated or layout changed",
            pool_data.len()
        );
        return;
    }

    let pool_base_ta = Pubkey::new_from_array(pool_data[139..171].try_into().unwrap());
    let pool_quote_ta = Pubkey::new_from_array(pool_data[171..203].try_into().unwrap());
    let coin_creator = Pubkey::new_from_array(pool_data[211..243].try_into().unwrap());

    let coin_creator_vault_authority =
        pda(&[b"creator_vault", coin_creator.as_ref()], &PUMPSWAP_PROGRAM);
    let coin_creator_vault_ata = ata(&coin_creator_vault_authority, &WSOL_MINT, &TOKEN_PROGRAM);

    let fee_recipient = PUMPSWAP_FEE_RECIPIENT;
    let fee_recipient_ata = ata(&fee_recipient, &WSOL_MINT, &TOKEN_PROGRAM);

    let user_vol = pda(
        &[b"user_volume_accumulator", payer.as_ref()],
        &PUMPSWAP_PROGRAM,
    );
    let pool_v2 = pda(&[b"pool-v2", base_mint.as_ref()], &PUMPSWAP_PROGRAM);

    println!("pumpswap slots:");
    println!("  base_mint: {base_mint}");
    println!("  pump_pool_authority: {pump_pool_authority}");
    println!("  pool: {pool}");
    println!("  pool_base_ta: {pool_base_ta}");
    println!("  pool_quote_ta: {pool_quote_ta}");
    println!("  coin_creator: {coin_creator}");
    println!("  coin_creator_vault_authority: {coin_creator_vault_authority}");
    println!("  coin_creator_vault_ata: {coin_creator_vault_ata}");
    println!("  fee_recipient: {fee_recipient}");
    println!("  fee_recipient_ata: {fee_recipient_ata}");
    println!("  user_vol: {user_vol}");
    println!("  pool_v2: {pool_v2}");

    let spec = tx_stamper::protocols::pumpswap::buy_v2_spec(payer, TokenProgram::Legacy);
    let tpl = match Template::compile(spec) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("pumpswap: compile error: {e}");
            return;
        }
    };

    let blockhash = match rpc.get_latest_blockhash() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("pumpswap: get_latest_blockhash error: {e}");
            return;
        }
    };

    let provider = PerProviderValues {
        slots: smallvec![
            ("tip_account".to_string(), JITO_TIP.into()),
            ("tip_lamports".to_string(), 1_000u64.into()),
        ],
    };

    let bundle = match tpl
        .stamp_bundle([provider])
        .set("pool", pool)
        .set("base_mint", base_mint)
        .set("pool_base_ta", pool_base_ta)
        .set("pool_quote_ta", pool_quote_ta)
        .set("fee_recipient", fee_recipient)
        .set("fee_recipient_ata", fee_recipient_ata)
        .set("coin_creator_vault_ata", coin_creator_vault_ata)
        .set("creator_vault_authority", coin_creator_vault_authority)
        .set("user_vol", user_vol)
        .set("pool_v2", pool_v2)
        .set("quote_amount_in", 10_000u64)
        .set("min_base_amount_out", 1u64)
        .set("cu_limit", 200_000u32)
        .set("cu_price", 100_000u64)
        .blockhash(blockhash)
        .sign(keypair)
    {
        Ok(b) => b,
        Err(e) => {
            eprintln!("pumpswap: stamp error: {e}");
            return;
        }
    };

    let stamped = bundle.reconstruct(0);
    let tx: VersionedTransaction = match bincode::deserialize(stamped.as_bytes()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("pumpswap: deserialize error: {e}");
            return;
        }
    };

    let result = rpc.simulate_transaction_with_config(&tx, sim_config());
    print_sim_result("PumpSwap", &result);
}

fn simulate_damm_v2(rpc: &RpcClient, keypair: &KeypairSigner) {
    let payer = keypair.pubkey();
    let base_mint = DAMM_V2_MINT;

    let wsol = WSOL_MINT;

    let (mint_a, mint_b) = if base_mint.to_bytes() < wsol.to_bytes() {
        (base_mint, wsol)
    } else {
        (wsol, base_mint)
    };
    let base_is_a = mint_a == base_mint;

    let pool = if let Ok(s) = std::env::var("DAMM_POOL") {
        Pubkey::from_str(&s).expect("DAMM_POOL parse")
    } else if base_mint.to_string() == "6VTv46gGe3e5jxEpCfwsNZUDMtJw7ssQif5PntXhHyAV" {
        solana_sdk::pubkey!("BcFQue8PDT48C1zvRQS6jaTFshW5KsiimvR1CRxyCjij")
    } else {
        panic!("no DAMM_POOL env and no hardcoded mapping for mint {base_mint}")
    };

    assert!(
        rpc.get_account(&pool).is_ok(),
        "DAMM v2 pool not found at {pool} for mint {base_mint}"
    );

    let vault_a = pda(&[b"token_vault", mint_a.as_ref(), pool.as_ref()], &DAMM_V2_PROGRAM);
    let vault_b = pda(&[b"token_vault", mint_b.as_ref(), pool.as_ref()], &DAMM_V2_PROGRAM);

    let user_input_mint = if base_is_a { wsol } else { base_mint };
    let user_output_mint = if base_is_a { base_mint } else { wsol };
    let user_input = ata(&payer, &user_input_mint, &TOKEN_PROGRAM);
    let user_output = ata(&payer, &user_output_mint, &TOKEN_PROGRAM);

    println!("damm_v2 slots:");
    println!("  base_mint: {base_mint}");
    println!("  mint_a: {mint_a}  mint_b: {mint_b}  base_is_a: {base_is_a}");
    println!("  pool: {pool}");
    println!("  vault_a: {vault_a}");
    println!("  vault_b: {vault_b}");
    println!("  user_input: {user_input}");
    println!("  user_output: {user_output}");

    let spec = tx_stamper::protocols::damm_v2::swap_spec(payer, TokenProgram::Legacy, base_is_a);
    let tpl = match Template::compile(spec) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("damm_v2: compile error: {e}");
            return;
        }
    };

    let blockhash = match rpc.get_latest_blockhash() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("damm_v2: get_latest_blockhash error: {e}");
            return;
        }
    };

    let provider = PerProviderValues {
        slots: smallvec![
            ("tip_account".to_string(), JITO_TIP.into()),
            ("tip_lamports".to_string(), 1_000u64.into()),
        ],
    };

    let bundle = match tpl
        .stamp_bundle([provider])
        .set("pool", pool)
        .set("user_input", user_input)
        .set("user_output", user_output)
        .set("token_a_vault", vault_a)
        .set("token_b_vault", vault_b)
        .set("mint", base_mint)
        .set("amount_in", 10_000u64)
        .set("min_amount_out", 1u64)
        .set("cu_limit", 200_000u32)
        .set("cu_price", 100_000u64)
        .blockhash(blockhash)
        .sign(keypair)
    {
        Ok(b) => b,
        Err(e) => {
            eprintln!("damm_v2: stamp error: {e}");
            return;
        }
    };

    let stamped = bundle.reconstruct(0);
    let tx: VersionedTransaction = match bincode::deserialize(stamped.as_bytes()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("damm_v2: deserialize error: {e}");
            return;
        }
    };

    let result = rpc.simulate_transaction_with_config(&tx, sim_config());
    print_sim_result("DAMM v2", &result);
}

fn parse_alt_addresses(data: &[u8]) -> Vec<Pubkey> {
    const HEADER: usize = 56;
    if data.len() < HEADER {
        return Vec::new();
    }
    data[HEADER..]
        .chunks_exact(32)
        .map(|c| Pubkey::new_from_array(c.try_into().unwrap()))
        .collect()
}

fn simulate_printr(rpc: &RpcClient, keypair: &KeypairSigner) {
    let payer = keypair.pubkey();
    let telecoin_mint = PRINTR_MINT;
    let wsol = WSOL_MINT;

    let dbc_config = pda(&[b"printr_config"], &PRINTR_PROGRAM);
    let damm_partner_config = PRINTR_PARTNER_CONFIG;

    let damm_position_nft_mint = pda(
        &[b"printr_damm_position_nft_mint", telecoin_mint.as_ref()],
        &PRINTR_PROGRAM,
    );
    let damm_position = pda(
        &[b"position", damm_position_nft_mint.as_ref()],
        &DAMM_V2_PROGRAM,
    );
    let damm_position_nft_account = pda(
        &[b"position_nft_account", damm_position_nft_mint.as_ref()],
        &DAMM_V2_PROGRAM,
    );
    let damm_pool_auth = damm_pool_authority();

    let (mint_a, mint_b) = if telecoin_mint.to_bytes() < wsol.to_bytes() {
        (telecoin_mint, wsol)
    } else {
        (wsol, telecoin_mint)
    };

    let damm_pool = pda(
        &[
            b"pool",
            damm_partner_config.as_ref(),
            mint_a.as_ref(),
            mint_b.as_ref(),
        ],
        &DAMM_V2_PROGRAM,
    );

    let damm_telecoin_vault = pda(
        &[b"token_vault", telecoin_mint.as_ref(), damm_pool.as_ref()],
        &DAMM_V2_PROGRAM,
    );
    let damm_quote_vault = pda(
        &[b"token_vault", wsol.as_ref(), damm_pool.as_ref()],
        &DAMM_V2_PROGRAM,
    );

    let dbc_pool = pda(&[b"pool", telecoin_mint.as_ref()], &DBC_PROGRAM);
    let dbc_pool_authority = pda(&[b"pool_authority"], &DBC_PROGRAM);
    let dbc_telecoin_vault = pda(
        &[b"token_vault", telecoin_mint.as_ref(), dbc_pool.as_ref()],
        &DBC_PROGRAM,
    );
    let dbc_quote_vault = pda(
        &[b"token_vault", wsol.as_ref(), dbc_pool.as_ref()],
        &DBC_PROGRAM,
    );
    let dbc_migration_metadata = pda(&[b"migration", telecoin_mint.as_ref()], &DBC_PROGRAM);

    let input_wallet = ata(&payer, &wsol, &TOKEN_PROGRAM);
    let output_wallet = ata(&payer, &telecoin_mint, &TOKEN_PROGRAM);

    println!("printr slots:");
    println!("  telecoin_mint: {telecoin_mint}");
    println!("  dbc_config: {dbc_config}");
    println!("  dbc_pool: {dbc_pool}");
    println!("  dbc_pool_authority: {dbc_pool_authority}");
    println!("  dbc_telecoin_vault: {dbc_telecoin_vault}");
    println!("  dbc_quote_vault: {dbc_quote_vault}");
    println!("  dbc_migration_metadata: {dbc_migration_metadata}");
    println!("  damm_partner_config: {damm_partner_config}");
    println!("  damm_pool: {damm_pool}");
    println!("  damm_pool_authority: {damm_pool_auth}");
    println!("  damm_telecoin_vault: {damm_telecoin_vault}");
    println!("  damm_quote_vault: {damm_quote_vault}");
    println!("  damm_position_nft_mint: {damm_position_nft_mint}");
    println!("  damm_position_nft_account: {damm_position_nft_account}");
    println!("  damm_position: {damm_position}");
    println!("  printr_lut: {PRINTR_LUT_ADDR}");
    println!("  input_wallet: {input_wallet}");
    println!("  output_wallet: {output_wallet}");

    let alt_data = match rpc.get_account_data(&PRINTR_LUT_ADDR) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("printr: failed to fetch ALT account: {e}");
            return;
        }
    };
    let printr_lut = tx_stamper::spec::lookup::LookupTable {
        address: PRINTR_LUT_ADDR,
        addresses: parse_alt_addresses(&alt_data),
    };

    let spec = tx_stamper::protocols::printr::buy_spec(payer, TokenProgram::Legacy, printr_lut);
    let tpl = match Template::compile(spec) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("printr: compile error: {e}");
            return;
        }
    };

    let blockhash = match rpc.get_latest_blockhash() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("printr: get_latest_blockhash error: {e}");
            return;
        }
    };

    let provider = PerProviderValues {
        slots: smallvec![
            ("tip_account".to_string(), JITO_TIP.into()),
            ("tip_lamports".to_string(), 1_000u64.into()),
        ],
    };

    let bundle = match tpl
        .stamp_bundle([provider])
        .set("input_wallet", input_wallet)
        .set("output_wallet", output_wallet)
        .set("mint", telecoin_mint)
        .set("dbc_pool_authority", dbc_pool_authority)
        .set("dbc_config", dbc_config)
        .set("dbc_pool", dbc_pool)
        .set("dbc_telecoin_vault", dbc_telecoin_vault)
        .set("dbc_quote_vault", dbc_quote_vault)
        .set("dbc_migration_metadata", dbc_migration_metadata)
        .set("damm_partner_config", damm_partner_config)
        .set("damm_pool", damm_pool)
        .set("damm_pool_authority", damm_pool_auth)
        .set("damm_telecoin_vault", damm_telecoin_vault)
        .set("damm_quote_vault", damm_quote_vault)
        .set("damm_position_nft_mint", damm_position_nft_mint)
        .set("damm_position_nft_account", damm_position_nft_account)
        .set("damm_position", damm_position)
        .set("amount", 10_000u64)
        .set("sqrt_price", u128::MAX)
        .set("cu_limit", 200_000u32)
        .set("cu_price", 100_000u64)
        .blockhash(blockhash)
        .sign(keypair)
    {
        Ok(b) => b,
        Err(e) => {
            eprintln!("printr: stamp error: {e}");
            return;
        }
    };

    let stamped = bundle.reconstruct(0);
    let tx: VersionedTransaction = match bincode::deserialize(stamped.as_bytes()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("printr: deserialize error: {e}");
            return;
        }
    };

    if let solana_sdk::message::VersionedMessage::V0(msg) = &tx.message {
        println!("printr static account_keys ({}):", msg.account_keys.len());
        for (i, k) in msg.account_keys.iter().enumerate() {
            println!("  [{i}] {k}");
        }
        println!("printr address_table_lookups ({}):", msg.address_table_lookups.len());
        for lut in &msg.address_table_lookups {
            println!("  table {} writable_indexes={:?} readonly_indexes={:?}",
                     lut.account_key, lut.writable_indexes, lut.readonly_indexes);
        }
    }

    let result = rpc.simulate_transaction_with_config(&tx, sim_config());
    print_sim_result("Printr", &result);
}

fn main() {
    let keypair = load_keypair_from_env();
    let rpc = RpcClient::new(RPC_URL.to_string());

    simulate_pumpfun(&rpc, &keypair);
    sleep(Duration::from_millis(500));

    simulate_pumpswap(&rpc, &keypair);
    sleep(Duration::from_millis(500));

    let damm_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        simulate_damm_v2(&rpc, &keypair);
    }));
    if let Err(e) = damm_result {
        let msg = e
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| e.downcast_ref::<&str>().copied())
            .unwrap_or("unknown panic");
        eprintln!("=== DAMM v2 ===\npanic: {msg}\n");
    }
    sleep(Duration::from_millis(500));

    let printr_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        simulate_printr(&rpc, &keypair);
    }));
    if let Err(e) = printr_result {
        let msg = e
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| e.downcast_ref::<&str>().copied())
            .unwrap_or("unknown panic");
        eprintln!("=== Printr ===\npanic: {msg}\n");
    }
}
