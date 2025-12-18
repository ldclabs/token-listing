import type { _SERVICE } from '$declarations/ic_auction/ic_auction.did'
import { idlFactory } from '$declarations/ic_auction/ic_auction.did.js'
import { createActor, type ActorSubclass } from '$lib/canisters/actors'
import { dynAgent } from '$lib/utils/auth'

export type AuctionService = ActorSubclass<_SERVICE>

export function icAuctionActor(canisterId: string): AuctionService {
  return createActor<_SERVICE>({
    canisterId,
    idlFactory,
    agent: dynAgent
  })
}
