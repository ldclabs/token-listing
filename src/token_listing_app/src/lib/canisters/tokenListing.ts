import type { _SERVICE } from '$declarations/token_listing_canister/token_listing_canister.did'
import { idlFactory } from '$declarations/token_listing_canister/token_listing_canister.did.js'
import { createActor } from '$lib/canisters/actors'
import { dynAgent } from '$lib/utils/auth'

export function tokenListingActor(canisterId: string) {
  return createActor<_SERVICE>({
    canisterId,
    idlFactory,
    agent: dynAgent
  })
}
