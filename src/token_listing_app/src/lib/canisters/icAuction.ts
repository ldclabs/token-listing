import type { _SERVICE } from '$declarations/ic_auction/ic_auction.did'
import { idlFactory } from '$declarations/ic_auction/ic_auction.did.js'
import { createActor } from '$lib/canisters/actors'
import { dynAgent } from '$lib/utils/auth'

function getCanisterId(): string {
  // vite-plugin-environment injects env at build-time; in TS we access via import.meta.env
  const id = ((import.meta as any).env?.CANISTER_ID_IC_AUCTION ??
    (globalThis as any)?.process?.env?.CANISTER_ID_IC_AUCTION) as
    | string
    | undefined
  if (!id) {
    throw new Error(
      'Missing CANISTER_ID_IC_AUCTION. Ensure dfx env vars are available.'
    )
  }
  return id
}

export function icAuctionActor(canisterId: string) {
  return createActor<_SERVICE>({
    canisterId,
    idlFactory,
    agent: dynAgent
  })
}
