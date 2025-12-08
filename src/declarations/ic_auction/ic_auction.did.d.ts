import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AuctionConfig {
  'min_amount' : bigint,
  'end_time' : bigint,
  'start_time' : bigint,
  'token_decimals' : number,
  'required_currency_raised' : bigint,
  'max_amount' : bigint,
  'total_supply' : bigint,
  'min_bid_duration' : bigint,
}
export interface AuctionInfo {
  'cumulative_demand_raised' : bigint,
  'total_amount' : bigint,
  'total_tokens_filled' : bigint,
  'timestamp' : bigint,
  'clearing_price' : bigint,
  'bidders_count' : bigint,
  'total_refunded' : bigint,
  'auction' : AuctionConfig,
  'is_graduated' : boolean,
  'cumulative_supply_released' : bigint,
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
  { 'Icp' : null } |
  { 'Sol' : null };
export interface DepositInput { 'txid' : string, 'sender' : string }
export interface InitArgs {
  'governance_canister' : [] | [Principal],
  'chain' : Chain,
  'key_name' : string,
}
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : BidInfo } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : Array<BidInfo> } |
  { 'Err' : string };
export type Result_3 = { 'Ok' : bigint } |
  { 'Err' : string };
export type Result_4 = { 'Ok' : string } |
  { 'Err' : string };
export type Result_5 = { 'Ok' : StateInfo } |
  { 'Err' : string };
export type Result_6 = { 'Ok' : WithdrawTxInfo } |
  { 'Err' : string };
export interface StateInfo {
  'token' : string,
  'sol_address' : string,
  'token_program_id' : [] | [string],
  'evm_address' : string,
  'governance_canister' : [] | [Principal],
  'chain' : Chain,
  'icp_address' : Principal,
  'chain_providers' : Array<string>,
  'currency_decimals' : number,
  'funds_recipient' : string,
  'tokens_recipient' : string,
  'currency' : string,
  'key_name' : string,
  'token_decimals' : number,
  'currency_program_id' : [] | [string],
}
export interface TokenInput {
  'decimals' : number,
  'token' : string,
  'program_id' : [] | [string],
  'recipient' : string,
}
export interface UpgradeArgs { 'governance_canister' : [] | [Principal] }
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
export interface _SERVICE {
  'admin_set_auction' : ActorMethod<[AuctionConfig], Result>,
  'admin_set_currency' : ActorMethod<[TokenInput], Result>,
  'admin_set_providers' : ActorMethod<[Array<string>], Result>,
  'admin_set_token' : ActorMethod<[TokenInput], Result>,
  'auction_info' : ActorMethod<[], [] | [AuctionInfo]>,
  'claim' : ActorMethod<[bigint], Result_1>,
  'claim_all' : ActorMethod<[], Result_2>,
  'deposit_currency' : ActorMethod<[DepositInput], Result_3>,
  'estimate_max_price' : ActorMethod<[bigint], bigint>,
  'evm_address' : ActorMethod<[[] | [Principal]], Result_4>,
  'get_grouped_bids' : ActorMethod<[[] | [bigint]], Array<[bigint, bigint]>>,
  'info' : ActorMethod<[], Result_5>,
  'my_bids' : ActorMethod<[], Result_2>,
  'sol_address' : ActorMethod<[[] | [Principal]], Result_4>,
  'submit_bid' : ActorMethod<[bigint, bigint], Result_1>,
  'validate_admin_set_currency' : ActorMethod<[TokenInput], Result_4>,
  'validate_admin_set_providers' : ActorMethod<[Array<string>], Result_4>,
  'validate_admin_set_token' : ActorMethod<[TokenInput], Result_4>,
  'withdraw_currency' : ActorMethod<[WithdrawInput], Result_6>,
  'withdraw_token' : ActorMethod<[WithdrawInput], Result_6>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
