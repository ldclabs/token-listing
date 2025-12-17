use solana_instruction::{AccountMeta, Instruction};

use super::types::Pubkey;

pub use spl_associated_token_account_interface::{
    address::get_associated_token_address_with_program_id as get_associated_token_address,
    instruction::create_associated_token_account_idempotent,
};

#[allow(clippy::too_many_arguments)]
pub fn transfer_checked_instruction(
    token_program_id: &Pubkey,
    source_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    authority_pubkey: &Pubkey, // The source account's owner/delegate.
    signer_pubkeys: &[&Pubkey],
    amount: u64,
    decimals: u8,
) -> Instruction {
    let mut data = Vec::with_capacity(10);
    // SPL token program "TransferChecked" instruction
    data.push(12);
    data.extend_from_slice(&amount.to_le_bytes());
    data.push(decimals);
    let mut accounts = Vec::with_capacity(4 + signer_pubkeys.len());
    accounts.push(AccountMeta::new(*source_pubkey, false));
    accounts.push(AccountMeta::new_readonly(*mint_pubkey, false));
    accounts.push(AccountMeta::new(*destination_pubkey, false));
    accounts.push(AccountMeta::new_readonly(
        *authority_pubkey,
        signer_pubkeys.is_empty(),
    ));
    for signer_pubkey in signer_pubkeys.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }
    Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    }
}
