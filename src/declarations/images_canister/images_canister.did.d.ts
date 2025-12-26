import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type CanisterArgs = { 'Upgrade' : InitArgs } |
  { 'Init' : InitArgs };
export interface ImageInput {
  'body' : Uint8Array | number[],
  'name' : string,
  'type' : string,
}
export interface ImageMetadata {
  'updated_at' : bigint,
  'updated_by' : Principal,
  'name' : string,
  'size' : bigint,
  'type' : string,
  'created_at' : bigint,
  'locations' : Array<string>,
}
export interface InitArgs { 'governance_canister' : [] | [Principal] }
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : ImageMetadata } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : StateInfo } |
  { 'Err' : string };
export type Result_3 = { 'Ok' : bigint } |
  { 'Err' : string };
export type Result_4 = { 'Ok' : string } |
  { 'Err' : string };
export interface StateInfo {
  'governance_canister' : [] | [Principal],
  'total_images' : bigint,
  'tokens_canister' : Principal,
}
export interface _SERVICE {
  'admin_set_tokens_canister' : ActorMethod<[Principal], Result>,
  'get_image_metadata' : ActorMethod<[bigint], Result_1>,
  'info' : ActorMethod<[], Result_2>,
  'update_image' : ActorMethod<[bigint, ImageInput], Result_3>,
  'validate_admin_set_tokens_canister' : ActorMethod<[Principal], Result_4>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
