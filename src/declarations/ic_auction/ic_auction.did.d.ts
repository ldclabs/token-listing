import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AuctionConfig {
  'min_amount' : bigint,
  'liquidity_pool_amount' : bigint,
  'end_time' : bigint,
  'start_time' : bigint,
  'required_currency_raised' : bigint,
  'max_amount' : bigint,
  'total_supply' : bigint,
  'min_bid_duration' : bigint,
}
export interface AuctionInfo {
  'cumulative_demand_raised' : bigint,
  'total_amount' : bigint,
  'total_tokens_filled' : bigint,
  'bids_count' : bigint,
  'timestamp' : bigint,
  'clearing_price' : bigint,
  'total_refunded' : bigint,
  'is_graduated' : boolean,
  'cumulative_supply_released' : bigint,
}
export interface AuctionSnapshot {
  'c' : bigint,
  'd' : bigint,
  'f' : bigint,
  's' : bigint,
  't' : bigint,
}
export interface BidInfo {
  'id' : bigint,
  'tokens_filled' : bigint,
  'outbid_time' : [] | [bigint],
  'acc_snapshot' : bigint,
  'create_time' : bigint,
  'claim_time' : bigint,
  'outbid_acc_snapshot' : [] | [bigint],
  'max_price' : bigint,
  'flow_rate' : bigint,
  'amount' : bigint,
  'refund' : bigint,
}
export type CanisterArgs = { 'Upgrade' : UpgradeArgs } |
  { 'Init' : InitArgs };
export type Chain = { 'Evm' : bigint } |
  { 'Icp' : bigint } |
  { 'Sol' : bigint };
export interface DepositInput { 'txid' : string, 'sender' : string }
export interface DepositTxInfo {
  'txid' : string,
  'user' : Principal,
  'sender' : string,
  'timestamp' : bigint,
  'amount' : bigint,
}
export type FinalizeKind = { 'CreatePool' : string } |
  { 'Transfer' : null };
export interface FinalizeOutput { 'txid' : string, 'pool_id' : string }
export interface InitArgs {
  'governance_canister' : [] | [Principal],
  'chain' : Chain,
  'key_name' : string,
}
export interface PayingResultInput {
  'result' : Uint8Array | number[],
  'signature' : Uint8Array | number[],
  'timestamp' : bigint,
}
export interface ProjectInput {
  'url' : string,
  'name' : string,
  'description' : string,
  'detail' : string,
  'restricted_countries' : Array<string>,
}
export type Result = { 'Ok' : [] | [FinalizeOutput] } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : string };
export type Result_10 = { 'Ok' : string } |
  { 'Err' : string };
export type Result_11 = { 'Ok' : X402PaymentOutput } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : WithdrawTxInfo } |
  { 'Err' : string };
export type Result_3 = { 'Ok' : BidInfo } |
  { 'Err' : string };
export type Result_4 = { 'Ok' : Array<BidInfo> } |
  { 'Err' : string };
export type Result_5 = { 'Ok' : bigint } |
  { 'Err' : string };
export type Result_6 = { 'Ok' : StateInfo } |
  { 'Err' : string };
export type Result_7 = { 'Ok' : Array<DepositTxInfo> } |
  { 'Err' : string };
export type Result_8 = { 'Ok' : UserInfo } |
  { 'Err' : string };
export type Result_9 = { 'Ok' : Array<WithdrawTxInfo> } |
  { 'Err' : string };
export interface StateInfo {
  'url' : string,
  'token' : string,
  'sol_address' : string,
  'token_program_id' : [] | [string],
  'evm_address' : string,
  'total_withdrawn_token' : bigint,
  'token_logo_url' : string,
  'token_symbol' : string,
  'governance_canister' : [] | [Principal],
  'chain' : Chain,
  'name' : string,
  'currency_name' : string,
  'total_bidders' : bigint,
  'icp_address' : Principal,
  'currency_symbol' : string,
  'description' : string,
  'detail' : string,
  'chain_providers' : Array<string>,
  'currency_decimals' : number,
  'funds_recipient' : string,
  'tokens_recipient' : string,
  'auction_config' : [] | [AuctionConfig],
  'currency' : string,
  'finalize_kind' : FinalizeKind,
  'key_name' : string,
  'token_decimals' : number,
  'total_deposited_currency' : bigint,
  'paying_public_keys' : Array<Uint8Array | number[]>,
  'token_name' : string,
  'finalize_output' : [] | [FinalizeOutput],
  'total_withdrawn_currency' : bigint,
  'restricted_countries' : Array<string>,
  'currency_program_id' : [] | [string],
  'currency_logo_url' : string,
}
export interface TokenInput {
  'decimals' : number,
  'token' : string,
  'name' : string,
  'program_id' : [] | [string],
  'recipient' : string,
  'logo_url' : string,
  'symbol' : string,
}
export interface UpgradeArgs { 'governance_canister' : [] | [Principal] }
export interface UserInfo {
  'token_amount' : bigint,
  'currency_amount' : bigint,
  'agreed_terms' : boolean,
  'timestamp' : bigint,
  'bound_addresses' : Array<string>,
}
export interface WithdrawInput { 'recipient' : string }
export interface WithdrawTxInfo {
  'id' : bigint,
  'kind' : number,
  'txid' : string,
  'user' : Principal,
  'recipient' : string,
  'timestamp' : bigint,
  'amount' : bigint,
}
export interface X402PaymentOutput {
  'x402' : Uint8Array | number[],
  'nonce' : string,
  'timestamp' : bigint,
}
export interface _SERVICE {
  'admin_finalize_auction' : ActorMethod<[], Result>,
  'admin_set_auction' : ActorMethod<[AuctionConfig], Result_1>,
  'admin_set_currency' : ActorMethod<[TokenInput], Result_1>,
  'admin_set_finalize' : ActorMethod<[FinalizeKind], Result_1>,
  'admin_set_paying_public_keys' : ActorMethod<[Array<string>], Result_1>,
  'admin_set_project' : ActorMethod<[ProjectInput], Result_1>,
  'admin_set_providers' : ActorMethod<[Array<string>], Result_1>,
  'admin_set_token' : ActorMethod<[TokenInput], Result_1>,
  'admin_setup_auction' : ActorMethod<[], Result_1>,
  'admin_sweep_currency' : ActorMethod<[], Result_2>,
  'admin_sweep_token' : ActorMethod<[], Result_2>,
  'auction_info' : ActorMethod<[], [] | [AuctionInfo]>,
  'claim' : ActorMethod<[bigint], Result_3>,
  'claim_all' : ActorMethod<[], Result_4>,
  'deposit_currency' : ActorMethod<[DepositInput], Result_5>,
  'estimate_max_price' : ActorMethod<[bigint], [bigint, bigint]>,
  'get_grouped_bids' : ActorMethod<[[] | [bigint]], Array<[bigint, bigint]>>,
  'get_snapshots' : ActorMethod<[bigint, bigint], Array<AuctionSnapshot>>,
  'info' : ActorMethod<[], Result_6>,
  'my_bids' : ActorMethod<[], Result_4>,
  'my_deposits' : ActorMethod<[], Result_7>,
  'my_info' : ActorMethod<[], Result_8>,
  'my_withdraws' : ActorMethod<[], Result_9>,
  'submit_bid' : ActorMethod<[bigint, bigint], Result_3>,
  'validate_admin_set_auction' : ActorMethod<[AuctionConfig], Result_10>,
  'validate_admin_set_currency' : ActorMethod<[TokenInput], Result_10>,
  'validate_admin_set_finalize' : ActorMethod<[FinalizeKind], Result_10>,
  'validate_admin_set_paying_public_keys' : ActorMethod<
    [Array<string>],
    Result_10
  >,
  'validate_admin_set_project' : ActorMethod<[ProjectInput], Result_10>,
  'validate_admin_set_providers' : ActorMethod<[Array<string>], Result_10>,
  'validate_admin_set_token' : ActorMethod<[TokenInput], Result_10>,
  'validate_empty_input' : ActorMethod<[], Result_10>,
  'withdraw_currency' : ActorMethod<[WithdrawInput], Result_2>,
  'withdraw_token' : ActorMethod<[WithdrawInput], Result_2>,
  'x402_bind_address' : ActorMethod<[PayingResultInput], Result_1>,
  'x402_deposit_currency' : ActorMethod<[PayingResultInput], Result_5>,
  'x402_payment' : ActorMethod<[bigint, boolean], Result_11>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
