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
  'total_refunded' : bigint,
  'auction' : AuctionConfig,
  'is_graduated' : boolean,
  'cumulative_supply_released' : bigint,
}
export type AuctionToken = { 'Evm' : string } |
  { 'Icp' : string } |
  { 'Sol' : string };
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
export interface InitArgs {
  'governance_canister' : [] | [Principal],
  'key_name' : string,
}
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : BidInfo } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : Array<BidInfo> } |
  { 'Err' : string };
export type Result_3 = { 'Ok' : string } |
  { 'Err' : string };
export type Result_4 = { 'Ok' : StateInfo } |
  { 'Err' : string };
export interface StateInfo {
  'token' : AuctionToken,
  'evm_address' : string,
  'svm_address' : string,
  'governance_canister' : [] | [Principal],
  'icp_address' : Principal,
  'chain_providers' : Array<string>,
  'funds_recipient' : string,
  'tokens_recipient' : string,
  'currency' : string,
  'key_name' : string,
}
export interface UpgradeArgs { 'governance_canister' : [] | [Principal] }
export interface _SERVICE {
  'admin_set_auction' : ActorMethod<[AuctionConfig], Result>,
  'admin_set_currency' : ActorMethod<[string, string], Result>,
  'admin_set_providers' : ActorMethod<[Array<string>], Result>,
  'admin_set_token' : ActorMethod<[AuctionToken, string], Result>,
  'auction_info' : ActorMethod<[], [] | [AuctionInfo]>,
  'claim' : ActorMethod<[bigint], Result_1>,
  'claim_all' : ActorMethod<[], Result_2>,
  'deposit_currency' : ActorMethod<[], Result>,
  'estimate_max_price' : ActorMethod<[bigint], bigint>,
  'evm_address' : ActorMethod<[[] | [Principal]], Result_3>,
  'get_grouped_bids' : ActorMethod<[[] | [bigint]], Array<[bigint, bigint]>>,
  'info' : ActorMethod<[], Result_4>,
  'my_bids' : ActorMethod<[], Result_2>,
  'submit_bid' : ActorMethod<[bigint, bigint], Result_1>,
  'svm_address' : ActorMethod<[[] | [Principal]], Result_3>,
  'validate_admin_set_currency' : ActorMethod<[string, string], Result_3>,
  'validate_admin_set_providers' : ActorMethod<[Array<string>], Result_3>,
  'validate_admin_set_token' : ActorMethod<[AuctionToken, string], Result_3>,
  'withdraw_currency' : ActorMethod<[], Result>,
  'withdraw_token' : ActorMethod<[], Result>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
