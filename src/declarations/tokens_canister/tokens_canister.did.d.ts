import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Announcement {
  'id' : bigint,
  'url' : [] | [string],
  'title' : string,
  'content' : string,
  'type' : string,
  'published_at' : bigint,
}
export type CanisterArgs = { 'Upgrade' : InitArgs } |
  { 'Init' : InitArgs };
export interface InitArgs { 'governance_canister' : [] | [Principal] }
export interface LinkItem { 'rel' : string, 'url' : string, 'name' : string }
export interface PayingResultInput {
  'result' : Uint8Array | number[],
  'signature' : Uint8Array | number[],
  'timestamp' : bigint,
}
export type Result = { 'Ok' : BigUint64Array | bigint[] } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : TokenProfile } |
  { 'Err' : string };
export type Result_3 = { 'Ok' : StateInfo } |
  { 'Err' : string };
export type Result_4 = { 'Ok' : Array<string> } |
  { 'Err' : string };
export type Result_5 = { 'Ok' : bigint } |
  { 'Err' : string };
export type Result_6 = { 'Ok' : string } |
  { 'Err' : string };
export type Result_7 = { 'Ok' : X402PaymentOutput } |
  { 'Err' : string };
export interface StateInfo {
  'governance_canister' : [] | [Principal],
  'x402_prices' : Array<[string, bigint]>,
  'x402_pay_to' : string,
  'total_incoming' : bigint,
  'total_tokens' : bigint,
  'x402_paying_public_keys' : Array<Uint8Array | number[]>,
}
export interface TokenMetadata {
  'decimals' : number,
  'external_url' : string,
  'name' : string,
  'description' : string,
  'links' : Array<LinkItem>,
  'locations' : Array<string>,
  'image' : string,
  'symbol' : string,
}
export interface TokenProfile {
  'id' : bigint,
  'status' : string,
  'updated_at' : bigint,
  'controllers' : Array<Principal>,
  'metadata' : TokenMetadata,
  'tags' : Array<string>,
  'created_at' : bigint,
  'verification' : VerificationBadge,
  'announcements' : Array<Announcement>,
}
export interface VerificationBadge {
  'methods' : Array<string>,
  'is_verified' : boolean,
  'verified_at' : bigint,
}
export interface X402PaymentOutput {
  'x402' : Uint8Array | number[],
  'nonce' : string,
  'timestamp' : bigint,
}
export interface _SERVICE {
  'admin_batch_register_tokens' : ActorMethod<[Array<TokenMetadata>], Result>,
  'admin_set_paying_public_keys' : ActorMethod<[Array<string>], Result_1>,
  'admin_set_payment_requirements_extra' : ActorMethod<[string], Result_1>,
  'admin_set_x402_pay_to' : ActorMethod<[string], Result_1>,
  'admin_set_x402_prices' : ActorMethod<[Array<[string, bigint]>], Result_1>,
  'admin_update_token_status' : ActorMethod<[bigint, string], Result_1>,
  'admin_update_token_tags' : ActorMethod<[bigint, Array<string>], Result_1>,
  'admin_update_token_verification_badge' : ActorMethod<
    [bigint, VerificationBadge],
    Result_1
  >,
  'check_permission' : ActorMethod<[bigint, Principal], Array<string>>,
  'get_token_profile' : ActorMethod<[bigint], Result_2>,
  'info' : ActorMethod<[], Result_3>,
  'list_tokens' : ActorMethod<
    [bigint, [] | [bigint]],
    Array<[bigint, TokenMetadata]>
  >,
  'my_txs' : ActorMethod<[], Result_4>,
  'query_token' : ActorMethod<[string], Array<[bigint, TokenMetadata]>>,
  'register_token' : ActorMethod<[TokenMetadata, PayingResultInput], Result_5>,
  'set_announcement' : ActorMethod<
    [bigint, Announcement, PayingResultInput],
    Result_1
  >,
  'update_token_controllers' : ActorMethod<
    [bigint, Array<Principal>],
    Result_1
  >,
  'update_token_metadata' : ActorMethod<[bigint, TokenMetadata], Result_1>,
  'validate_admin_set_paying_public_keys' : ActorMethod<
    [Array<string>],
    Result_6
  >,
  'validate_admin_set_payment_requirements_extra' : ActorMethod<
    [string],
    Result_6
  >,
  'validate_admin_set_x402_pay_to' : ActorMethod<[string], Result_6>,
  'validate_admin_set_x402_prices' : ActorMethod<
    [Array<[string, bigint]>],
    Result_6
  >,
  'x402_payment' : ActorMethod<[string], Result_7>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
