import { IS_LOCAL } from '$lib/constants'
import {
  AnonymousIdentity,
  HttpAgent,
  type HttpAgentOptions,
  type HttpAgentRequest,
  type Identity
} from '@dfinity/agent'
import { AuthClient, IdbStorage } from '@dfinity/auth-client'
import { DelegationChain } from '@dfinity/identity'
import type { Principal } from '@dfinity/principal'

export const EXPIRATION_MS = 1000 * 60 * 60 // 1 hour

export class IdentityEx implements Identity {
  expiredHook: (() => void) | null = null

  constructor(
    public readonly id: Identity,
    public readonly expiration: number, // in milliseconds
    public readonly username: string = '' // this is name identity if username exists
  ) {
    this.id = id
    this.expiration = id.getPrincipal().isAnonymous()
      ? Number.MAX_SAFE_INTEGER
      : expiration
    this.username = username
  }

  get isExpired() {
    return Date.now() >= this.expiration - 1000 * 60 * 5 // 3 minutes before expiration
  }

  get isAuthenticated() {
    return !this.id.getPrincipal().isAnonymous() && !this.isExpired
  }

  getPrincipal(): Principal {
    return this.id.getPrincipal()
  }

  transformRequest(request: HttpAgentRequest): Promise<unknown> {
    if (this.isExpired) {
      if (this.expiredHook) this.expiredHook()
      throw new Error('Identity expired, please sign in again')
    }

    return this.id.transformRequest(request)
  }
}

export const anonymousIdentity = new IdentityEx(new AnonymousIdentity(), 0)

// II auth storage
const storage = new IdbStorage()

// should create a new authClient for each login
export function createAuthClient(): Promise<AuthClient> {
  return AuthClient.create({
    keyType: 'Ed25519',
    idleOptions: {
      disableIdle: true,
      disableDefaultIdleCallback: true
    }
  })
}

export async function loadIdentity(
  client?: AuthClient
): Promise<IdentityEx | null> {
  const authClient = client || (await createAuthClient())
  const authenticated = await authClient.isAuthenticated()

  // Not authenticated therefore we provide no identity as a result
  if (authenticated) {
    const expiration = await tryGetDelegationExpiration()
    return new IdentityEx(authClient.getIdentity(), expiration)
  }

  return null
}

const KEY_STORAGE_DELEGATION = 'delegation'
async function tryGetDelegationExpiration(): Promise<number> {
  let expiration = Date.now() + EXPIRATION_MS

  try {
    const delegation = await storage.get(KEY_STORAGE_DELEGATION)
    if (delegation) {
      const chain = DelegationChain.fromJSON(delegation)
      expiration = getDelegationExpiration(chain)
    }
  } catch (e) {}

  return expiration
}

function getDelegationExpiration(chain: DelegationChain): number {
  let expiration = Date.now() + EXPIRATION_MS
  for (const { delegation } of chain.delegations) {
    // prettier-ignore
    const ex = Number(delegation.expiration / BigInt(1000000))
    if (ex < expiration) {
      expiration = ex
    }
  }
  return expiration
}

export class AuthAgent extends HttpAgent {
  private _id: Identity
  constructor(options: { identity: Identity } & HttpAgentOptions) {
    super(options)
    this._id = options.identity
  }

  get id() {
    return this._id
  }

  isAnonymous() {
    return this._id.getPrincipal().isAnonymous()
  }

  setIdentity(id: Identity) {
    this._id = id
    super.replaceIdentity(id)
  }
}

export function createAgent(identity: Identity): AuthAgent {
  return new AuthAgent({
    identity,
    host: IS_LOCAL ? 'http://localhost:4943/' : 'https://icp-api.io',
    verifyQuerySignatures: false,
    shouldFetchRootKey: IS_LOCAL
  })
}

export const dynAgent = createAgent(anonymousIdentity)
export const anonAgent = new AuthAgent({
  identity: anonymousIdentity,
  host: 'https://icp-api.io',
  verifyQuerySignatures: false,
  shouldFetchRootKey: IS_LOCAL
})
