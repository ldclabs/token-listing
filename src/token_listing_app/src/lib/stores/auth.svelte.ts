import { IS_LOCAL } from '$lib/constants'
import {
  anonymousIdentity,
  createAuthClient,
  dynAgent,
  EXPIRATION_MS,
  IdentityEx,
  loadIdentity
} from '$lib/utils/auth'
import { type Signer } from '$lib/utils/fetcher'
import { popupCenter } from '$lib/utils/window'
import { type AuthClient } from '@dfinity/auth-client'
import { type DelegationIdentity } from '@dfinity/identity'
import {
  bytesToBase64Url,
  deterministicEncode,
  signArbitrary
} from '@ldclabs/ic-auth'

const IDENTITY_PROVIDER = IS_LOCAL
  ? 'http://rdmx6-jaaaa-aaaaa-aaadq-cai.localhost:4943'
  : 'https://id.ai'
const authClientPromise = createAuthClient()
let authClient: AuthClient | null = null
authClientPromise.then((client) => {
  authClient = client
})

export const EventLogin = 'Login'

export class AuthStore extends EventTarget implements Signer {
  static async init() {
    await authClientPromise
    const identity = await loadIdentity(authClient!)
    // await fetchRootKey()

    if (identity) {
      AuthStore.#login(identity)
    }

    authStore.#initPromiseResolve()
  }

  static #login(identity: IdentityEx) {
    identity.expiredHook = () => authStore.logout()
    dynAgent.setIdentity(identity)
    authStore.#identity = identity
    authStore.dispatchEvent(
      new CustomEvent(EventLogin, { detail: identity.getPrincipal().toText() })
    )
  }

  #identity = $state<IdentityEx>(anonymousIdentity)
  #initPromiseResolve: () => void = () => {}
  #initPromise

  constructor() {
    super()
    this.#initPromise = new Promise(
      (resolve) => (this.#initPromiseResolve = resolve as () => void)
    )
  }

  get identity() {
    return this.#identity
  }

  async ready() {
    return this.#initPromise
  }

  async signHash(hash: Uint8Array): Promise<string | null> {
    if (!this.#identity.isAuthenticated) {
      return null
    }

    const sig = await signArbitrary(
      this.#identity.id as any as DelegationIdentity,
      hash
    )
    delete sig.h
    return 'ICP ' + bytesToBase64Url(deterministicEncode(sig))
  }

  signIn(identityProvider = IDENTITY_PROVIDER): Promise<null> {
    if (!authClient) {
      return Promise.reject(new Error('AuthClient not initialized'))
    }

    let resolve: (rt: null) => void, reject: (reason?: string) => void
    const promise = new Promise<null>((_resolve, _reject) => {
      resolve = _resolve
      reject = _reject
    })

    // 确保 window.open 在用户点击事件的同步调用栈内被触发，以避免被浏览器拦截
    // Important: authClientPromise should be resolved here
    // https://ffan0811.medium.com/window-open-returns-null-in-safari-and-firefox-after-allowing-pop-up-on-the-browser-4e4e45e7d926
    authClient.login({
      maxTimeToLive: BigInt(EXPIRATION_MS) * 1000000n,
      identityProvider,
      onSuccess: (msg) => {
        const authnMethod = msg.authnMethod
        const authnOrigin = location.origin
        console.log(`Login successful using ${authnMethod} from ${authnOrigin}`)

        const identity = new IdentityEx(
          authClient!.getIdentity(),
          Date.now() + EXPIRATION_MS
        )

        AuthStore.#login(identity)
        resolve(null)
      },
      onError: (err) => {
        console.error(err)
        reject(err)
      },
      windowOpenerFeatures: popupCenter({
        width: 576,
        height: 625
      })
    })

    return promise
  }

  async logout(url: string = '/') {
    this.#identity = anonymousIdentity
    dynAgent.setIdentity(anonymousIdentity)

    await authClient!.logout()
    if (url) {
      window.location.assign(url) // force reload to clear all auth state!!
    }
  }
}

export const authStore = new AuthStore()
