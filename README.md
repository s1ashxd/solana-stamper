# tx-stamper

V0 Solana transaction templates with byte-level re-stamping.

## Quickstart

```rust
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use tx_stamper::signer::{KeypairSigner, Signer};
use tx_stamper::spec::account::Acc;
use tx_stamper::spec::data::DataSpec;
use tx_stamper::spec::instruction::InstructionSpec;
use tx_stamper::spec::{MessageVersion, TemplateSpec};
use tx_stamper::template::Template;

let signer = KeypairSigner::from_bytes(&[42u8; 32]);
let spec = TemplateSpec::new(signer.pubkey(), MessageVersion::V0).ix(
    InstructionSpec::new(Pubkey::default())
        .account(Acc::payer())
        .account(Acc::slot_w("recipient"))
        .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")),
);
let template = Template::compile(spec)?;

let stamped = template.stamp()
    .set("recipient", Pubkey::new_unique())
    .set("amount", 1_000u64)
    .blockhash(Hash::new_unique())
    .sign(&signer)?;

let bytes = stamped.as_bytes();
let base64 = stamped.to_base64();
```

## Protocol presets

Pre-built `TemplateSpec` builders for common Solana protocols. Each protocol
is gated behind a Cargo feature.

| Protocol | Functions | Feature |
| --- | --- | --- |
| PumpFun | `buy_v2_spec`, `sell_v2_spec` | `pumpfun` |
| PumpSwap | `buy_v2_spec` | `pumpswap` |
| DAMM v2 | `swap_spec` | `damm-v2` |
| Printr (with ALT) | `buy_spec`, `sell_spec` | `printr` |

Enable individual features or use `all-protocols` for the full set:

```toml
[dependencies]
tx-stamper = { version = "0.1", features = ["pumpfun", "pumpswap"] }
```

Each preset configures the canonical program ID, fixed accounts, discriminator,
and a compute-budget + tip-transfer prefix. Per-trade values (mint, pool, vaults,
amounts) are exposed as named slots set at stamp time:

```rust
use tx_stamper::protocols::common::TokenProgram;
use tx_stamper::protocols::pumpfun::buy_v2_spec;
use tx_stamper::template::Template;

let spec = buy_v2_spec(signer.pubkey(), TokenProgram::Legacy);
let template = Template::compile(spec)?;

let stamped = template.stamp()
    .set("mint", mint_pubkey)
    .set("amount", 1_000_000u64)
    .blockhash(recent_blockhash)
    .sign(&signer)?;
```

## Status

Early development. API unstable.

## License

MIT.
