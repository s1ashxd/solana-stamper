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

## Status

Early development. API unstable.

## License

MIT.
