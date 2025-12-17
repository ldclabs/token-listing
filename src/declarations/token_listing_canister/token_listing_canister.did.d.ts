import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type AuctionId = { 'Evm' : string } |
  { 'Icp' : string } |
  { 'Sol' : string };
export interface AuctionInfo {
  'id' : AuctionId,
  'url' : string,
  'token' : string,
  'token_logo_url' : string,
  'total_demand_raised' : bigint,
  'token_symbol' : string,
  'chain' : Chain,
  'name' : string,
  'currency_name' : string,
  'update_time' : bigint,
  'currency_symbol' : string,
  'description' : string,
  'end_time' : bigint,
  'bids_count' : bigint,
  'total_supply_released' : bigint,
  'start_time' : bigint,
  'currency_decimals' : number,
  'currency' : string,
  'clearing_price' : bigint,
  'token_decimals' : number,
  'token_name' : string,
  'is_graduated' : boolean,
  'currency_logo_url' : string,
}
export type CanisterArgs = { 'Upgrade' : UpgradeArgs } |
  { 'Init' : InitArgs };
export type Chain = { 'Evm' : bigint } |
  { 'Icp' : bigint } |
  { 'Sol' : bigint };
export interface InitArgs {
  'governance_canister' : [] | [Principal],
  'key_name' : string,
}
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : StateInfo } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : string } |
  { 'Err' : string };
export interface StateInfo {
  'sol_address' : string,
  'evm_address' : string,
  'governance_canister' : [] | [Principal],
  'daos' : Array<Principal>,
  'auctions' : Array<AuctionId>,
  'icp_address' : Principal,
  'chain_providers' : Array<[Chain, Array<string>]>,
  'key_name' : string,
  'paying_public_keys' : Array<Uint8Array | number[]>,
  'storages' : Array<Principal>,
}
export interface UpgradeArgs { 'governance_canister' : [] | [Principal] }
export interface _SERVICE {
  'admin_set_auction' : ActorMethod<[AuctionInfo], Result>,
  'admin_set_paying_public_keys' : ActorMethod<[Array<string>], Result>,
  'admin_set_providers' : ActorMethod<[Chain, Array<string>], Result>,
  'get_auction' : ActorMethod<[[] | [AuctionId]], [] | [AuctionInfo]>,
  'info' : ActorMethod<[], Result_1>,
  'list_auctions' : ActorMethod<[bigint, [] | [AuctionId]], Array<AuctionInfo>>,
  'validate_admin_set_auction' : ActorMethod<[AuctionInfo], Result_2>,
  'validate_admin_set_paying_public_keys' : ActorMethod<
    [Array<string>],
    Result_2
  >,
  'validate_admin_set_providers' : ActorMethod<
    [Chain, Array<string>],
    Result_2
  >,
  'validate_empty_input' : ActorMethod<[], Result_2>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
