use proptest::prelude::*;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;

use tx_stamper::signer::{KeypairSigner, Signer};
use tx_stamper::spec::account::Acc;
use tx_stamper::spec::data::DataSpec;
use tx_stamper::spec::instruction::InstructionSpec;
use tx_stamper::spec::{MessageVersion, TemplateSpec};
use tx_stamper::template::Template;

proptest! {
    #[test]
    fn random_amount_round_trips(amount in any::<u64>()) {
        let signer = KeypairSigner::from_bytes(&[71u8; 32]);
        let payer = signer.pubkey();
        let recipient = Pubkey::new_unique();
        let blockhash = Hash::new_from_array([99u8; 32]);

        let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(
            InstructionSpec::new(Pubkey::default())
                .account(Acc::payer())
                .account(Acc::slot_w("recipient"))
                .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")));
        let tpl = Template::compile(spec).unwrap();
        let stamped = tpl.stamp()
            .set("recipient", recipient)
            .set("amount", amount)
            .blockhash(blockhash)
            .sign(&signer).unwrap();

        let tx: solana_sdk::transaction::VersionedTransaction =
            bincode::deserialize(stamped.as_bytes()).unwrap();
        let solana_sdk::message::VersionedMessage::V0(msg) = &tx.message else { unreachable!() };
        let buy_ix = msg.instructions.last().unwrap();
        let recovered = u64::from_le_bytes(buy_ix.data[4..12].try_into().unwrap());
        prop_assert_eq!(recovered, amount);
    }
}
