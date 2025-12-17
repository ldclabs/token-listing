import type { _SERVICE } from '$declarations/ic_auction/ic_auction.did'
import { idlFactory } from '$declarations/ic_auction/ic_auction.did.js'
import { createActor } from '$lib/canisters/actors'
import { dynAgent } from '$lib/utils/auth'

export function icAuctionActor(canisterId: string) {
  return createActor<_SERVICE>({
    canisterId,
    idlFactory,
    agent: dynAgent
  })
}
