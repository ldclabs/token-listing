import { IS_LOCAL } from '$lib/constants'
import {
  anonymousIdentity,
  createAuthClient,
  dynAgent,
  EXPIRATION_MS,
  IdentityEx,
  loadIdentity
} from '$lib/utils/auth'
import { popupCenter } from '$lib/utils/window'

const IDENTITY_PROVIDER = IS_LOCAL
  ? 'http://rdmx6-jaaaa-aaaaa-aaadq-cai.localhost:4943'
  : 'https://id.ai'

const authClientPromise = createAuthClient()

class AuthStore {
  static async init() {
    // Fetch the root key for local development
    if (IS_LOCAL) {
      await Promise.all([dynAgent.fetchRootKey(), dynAgent.syncTime()])
    }
    const authClient = await authClientPromise
    const identity = await loadIdentity(authClient)
    if (identity) {
      identity.expiredHook = () => authStore.logout()
      dynAgent.setIdentity(identity)
      authStore.#identity = identity
    }
  }

  #identity = $state<IdentityEx>(anonymousIdentity)

  get identity() {
    return this.#identity
  }

  signIn(identityProvider = IDENTITY_PROVIDER) {
    return new Promise<void>(async (resolve, reject) => {
      // Important: authClientPromise should be resolved here
      // https://ffan0811.medium.com/window-open-returns-null-in-safari-and-firefox-after-allowing-pop-up-on-the-browser-4e4e45e7d926
      const authClient = await authClientPromise
      await authClient.login({
        maxTimeToLive: BigInt(EXPIRATION_MS) * 1000000n,
        identityProvider,
        onSuccess: (msg) => {
          const authnMethod = msg.authnMethod
          const authnOrigin = location.origin
          console.log(
            `Login successful using ${authnMethod} from ${authnOrigin}`
          )

          const identity = new IdentityEx(
            authClient.getIdentity(),
            Date.now() + EXPIRATION_MS
          )
          identity.expiredHook = () => this.logout()
          this.#identity = identity
          dynAgent.setIdentity(identity)
          resolve()
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
    })
  }

  async logout(url?: string) {
    this.#identity = anonymousIdentity
    dynAgent.setIdentity(anonymousIdentity)
    const authClient = await authClientPromise
    await authClient.logout()
    url && window.location.assign(url) // force reload to clear all auth state!!
  }
}

export const authStore = new AuthStore()

AuthStore.init().catch((err) => {
  console.error('Failed to initialize AuthStore', err)
})
