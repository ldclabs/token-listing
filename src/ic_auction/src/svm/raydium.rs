use borsh::BorshSerialize;
use solana_instruction::{AccountMeta, Instruction};
use solana_program::{config::program, pubkey::Pubkey, sysvar};

use super::{constants, spl::get_associated_token_address};
use crate::helper::sha256;

// https://github.com/raydium-io/raydium-cp-swap/blob/master/client/src/instructions/amm_instructions.rs

const AMM_CONFIG_SEED: &str = "amm_config";
const POOL_SEED: &str = "pool";
const POOL_VAULT_SEED: &str = "pool_vault";
const POOL_LP_MINT_SEED: &str = "pool_lp_mint";
const OBSERVATION_SEED: &str = "observation";
const AUTH_SEED: &str = "vault_and_lp_mint_auth_seed";

//  devnet: DRaycpLY18LhpbydsBWbVJtxpNv9oXPgjRSfpF2bWpYb
pub const PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");
pub const PROGRAM_ID_DEV: Pubkey =
    Pubkey::from_str_const("DRaycpLY18LhpbydsBWbVJtxpNv9oXPgjRSfpF2bWpYb");

#[derive(BorshSerialize)]
struct CreateAmmConfigArgs {
    index: u16,
    trade_fee_rate: u64,
    protocol_fee_rate: u64,
    fund_fee_rate: u64,
    create_pool_fee: u64,
    creator_fee_rate: u64,
}

#[derive(BorshSerialize)]
struct InitializeArgs {
    init_amount_0: u64,
    init_amount_1: u64,
    open_time: u64,
}

fn get_function_hash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(&sha256(preimage.as_bytes())[..8]);
    sighash
}

/// 构建 CreateAmmConfig 指令
pub fn build_create_amm_config_ix(
    program_id: Pubkey,
    owner: Pubkey,
    trade_fee_rate: u64,
    protocol_fee_rate: u64,
) -> (Instruction, Pubkey) {
    let index = 0u16;
    let (amm_config, _) = Pubkey::find_program_address(
        &[AMM_CONFIG_SEED.as_bytes(), &index.to_be_bytes()],
        &program_id,
    );

    let accounts = vec![
        AccountMeta::new(owner, true),
        AccountMeta::new(amm_config, false),
        AccountMeta::new_readonly(constants::system_program::id(), false),
    ];

    let args = CreateAmmConfigArgs {
        index,
        trade_fee_rate,
        protocol_fee_rate,
        fund_fee_rate: 0,
        create_pool_fee: 0,
        creator_fee_rate: 0,
    };

    let mut data = get_function_hash("global", "create_amm_config").to_vec();
    data.append(&mut borsh::to_vec(&args).unwrap());

    (
        Instruction {
            program_id: program_id,
            accounts,
            data,
        },
        amm_config,
    )
}

/// 构建 Initialize Pool 指令
#[allow(clippy::too_many_arguments)]
pub fn build_initialize_pool_ix(
    program_id: Pubkey,
    creator: Pubkey,
    amm_config: Pubkey,
    token0_mint: Pubkey,
    token1_mint: Pubkey,
    token0_program: Pubkey,
    token1_program: Pubkey,
    creator_token_0_account: Pubkey,
    creator_token_1_account: Pubkey,
    create_pool_fee: Pubkey,
    init_amount_0: u64,
    init_amount_1: u64,
    open_time: u64, // unix timestamp seconds
    pool_id: Option<Pubkey>,
) -> (Instruction, PoolDerived) {
    // 1. 计算 PDA
    let pool_id = pool_id.unwrap_or_else(|| {
        Pubkey::find_program_address(
            &[
                POOL_SEED.as_bytes(),
                amm_config.as_ref(),
                token0_mint.as_ref(),
                token1_mint.as_ref(),
            ],
            &program_id,
        )
        .0
    });

    let (authority, _) = Pubkey::find_program_address(&[AUTH_SEED.as_bytes()], &program_id);
    let (token0_vault, _) = Pubkey::find_program_address(
        &[
            POOL_VAULT_SEED.as_bytes(),
            pool_id.as_ref(),
            token0_mint.as_ref(),
        ],
        &program_id,
    );
    let (token1_vault, _) = Pubkey::find_program_address(
        &[
            POOL_VAULT_SEED.as_bytes(),
            pool_id.as_ref(),
            token1_mint.as_ref(),
        ],
        &program_id,
    );
    let (lp_mint, _) = Pubkey::find_program_address(
        &[POOL_LP_MINT_SEED.as_bytes(), pool_id.as_ref()],
        &program_id,
    );
    let (observation_state, _) = Pubkey::find_program_address(
        &[OBSERVATION_SEED.as_bytes(), pool_id.as_ref()],
        &program_id,
    );

    let creator_lp_token =
        get_associated_token_address(&creator, &lp_mint, &constants::spl_token::id());

    // 2. 构造 AccountMeta 列表 (顺序必须严格一致)
    let accounts = vec![
        AccountMeta::new(creator, true),                  // 0. creator
        AccountMeta::new_readonly(amm_config, false),     // 1. amm_config
        AccountMeta::new_readonly(authority, false),      // 2. authority
        AccountMeta::new(pool_id, false),                 // 3. pool_state
        AccountMeta::new_readonly(token0_mint, false),    // 4. token_0_mint
        AccountMeta::new_readonly(token1_mint, false),    // 5. token_1_mint
        AccountMeta::new(lp_mint, false),                 // 6. lp_mint
        AccountMeta::new(creator_token_0_account, false), // 7. creator_token_0
        AccountMeta::new(creator_token_1_account, false), // 8. creator_token_1
        AccountMeta::new(creator_lp_token, false),        // 9. creator_lp_token
        AccountMeta::new(token0_vault, false),            // 10. token_0_vault
        AccountMeta::new(token1_vault, false),            // 11. token_1_vault
        AccountMeta::new(create_pool_fee, false),         // 12. create_pool_fee
        AccountMeta::new(observation_state, false),       // 13. observation_state
        AccountMeta::new_readonly(constants::spl_token::id(), false), // 14. token_program
        AccountMeta::new_readonly(token0_program, false), // 15. token_0_program
        AccountMeta::new_readonly(token1_program, false), // 16. token_1_program
        AccountMeta::new_readonly(constants::spl_associated_token_account::id(), false), // 17. associated_token_program
        AccountMeta::new_readonly(constants::system_program::id(), false), // 18. system_program
        AccountMeta::new_readonly(sysvar::rent::id(), false),              // 19. rent
    ];

    // 3. 构造指令数据
    let args = InitializeArgs {
        init_amount_0,
        init_amount_1,
        open_time,
    };
    let mut data = get_function_hash("global", "initialize").to_vec();
    data.append(&mut borsh::to_vec(&args).unwrap());

    let ix = Instruction {
        program_id: program_id,
        accounts,
        data,
    };

    (
        ix,
        PoolDerived {
            pool_id,
            lp_mint,
            token0_vault,
            token1_vault,
            observation_state,
            authority,
            creator_lp_token,
        },
    )
}

#[allow(unused)]
pub struct PoolDerived {
    pub pool_id: Pubkey,
    pub lp_mint: Pubkey,
    pub token0_vault: Pubkey,
    pub token1_vault: Pubkey,
    pub observation_state: Pubkey,
    pub authority: Pubkey,
    pub creator_lp_token: Pubkey,
}
