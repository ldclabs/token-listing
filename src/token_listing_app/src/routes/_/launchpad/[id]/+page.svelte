<script lang="ts">
  import { page } from '$app/state'
  import type {
    AuctionConfig,
    AuctionInfo,
    BidInfo,
    StateInfo,
    UserInfo
  } from '$declarations/ic_auction/ic_auction.did'
  import { icAuctionActor } from '$lib/canisters/icAuction'
  import { tokenListingActor } from '$lib/canisters/tokenListing'
  import CanisterModal from '$lib/components/CanisterModal.svelte'
  import CCAModal from '$lib/components/CCAModal.svelte'
  import Header from '$lib/components/Header.svelte'
  import UserFundsModal from '$lib/components/UserFundsModal.svelte'
  import { TOKEN_LISTING } from '$lib/constants'
  import ArrowRightUpLine from '$lib/icons/arrow-right-up-line.svelte'
  import InformationLine from '$lib/icons/information-line.svelte'
  import Settings4Line from '$lib/icons/settings-4-line.svelte'
  import { authStore, EventLogin } from '$lib/stores/auth.svelte'
  import { showModal } from '$lib/stores/modal.svelte'
  import { toastRun, triggerToast } from '$lib/stores/toast.svelte'
  import { unwrapOption, unwrapResult } from '$lib/types/result'
  import Button from '$lib/ui/Button.svelte'
  import Spinner from '$lib/ui/Spinner.svelte'
  import {
    getAccountUrl,
    getSwapUrl,
    getTokenUrl,
    getTxUrl
  } from '$lib/utils/chain'
  import { formatDatetime, parseUnits, pruneAddress } from '$lib/utils/helper'
  import { renderContent } from '$lib/utils/markdown'
  import {
    ICPToken,
    PANDAToken,
    TokenDisplay,
    type TokenInfo
  } from '$lib/utils/token'
  import { Principal } from '@dfinity/principal'
  import { onDestroy, onMount, tick } from 'svelte'

  const listingActor = tokenListingActor(TOKEN_LISTING)

  let canister = $state('kfzgd-diaaa-aaaap-an56q-cai')
  const actor = $derived(icAuctionActor(canister))

  let isListed = $state(true)
  let stateInfo = $state<StateInfo | null>(null)
  let auctionCfg = $state<AuctionConfig | null>(null)
  let auctionInfo = $state<AuctionInfo | null>(null)
  let myInfo = $state<UserInfo>({
    token_amount: 0n, // token balance
    currency_amount: 0n, // currency balance
    agreed_terms: false,
    timestamp: 0n,
    bound_addresses: []
  })
  let grouped = $state<Array<[bigint, bigint]>>([])
  let tokenInfo = $state<TokenInfo>(PANDAToken)
  let currencyInfo = $state<TokenInfo>(ICPToken)

  const tokenDisplay = $derived.by(() => new TokenDisplay(tokenInfo, 0n))
  const currencyDisplay = $derived.by(() => new TokenDisplay(currencyInfo, 0n))

  let myBids = $state<BidInfo[]>([])

  // Inputs
  let floorPrice = $state(ICPToken.one)
  let bidAmount = $state('')
  let bidMaxPrice = $state('')

  let isDetailExpanded = $state(false)
  const detailText = $derived.by(() =>
    (stateInfo?.detail || stateInfo?.description || '').trim()
  )
  const isDetailLong = $derived.by(() => {
    if (!detailText) return false
    return detailText.length > 280 || detailText.split('\n').length > 6
  })

  $effect(() => {
    // Reset to collapsed when project/description changes
    detailText
    isDetailExpanded = false
  })

  let floorGroupedPrecision = $derived.by(() => {
    const val = Math.max(floorPrice.toString().length - 1, 1)
    return 10n ** BigInt(val)
  })
  let groupedPrecision = $state('')

  const phase = $derived.by(() => {
    if (!auctionCfg) return 'unconfigured' as const
    if (!auctionInfo) return 'configured' as const

    const n = BigInt(Date.now())
    if (n < auctionCfg.start_time) return 'prebid' as const
    if (n + auctionCfg.min_bid_duration < auctionCfg.end_time)
      return 'bidding' as const
    if (n < auctionCfg.end_time) return 'ending' as const
    return 'ended' as const
  })

  function getFinalizeKind(): [string, string] {
    if (!stateInfo) return ['—', '']
    if ('Transfer' in stateInfo.finalize_kind) {
      return [
        `Transfer to ${pruneAddress(stateInfo.funds_recipient, false)}`,
        getAccountUrl(stateInfo.chain, stateInfo.funds_recipient)
      ]
    } else if ('CreatePool' in stateInfo.finalize_kind) {
      return [
        `Create pool on ${stateInfo.finalize_kind.CreatePool}`,
        getSwapUrl(stateInfo.finalize_kind.CreatePool)
      ]
    } else {
      return ['Unknown', '']
    }
  }

  function progress(cfg: AuctionConfig | null): number {
    if (!cfg) return 0
    const n = BigInt(Date.now())
    const start = cfg.start_time
    const end = cfg.end_time
    if (end <= start) return 0
    if (n <= start) return 0
    if (n >= end) return 1
    const done = Number(n - start)
    const total = Number(end - start)
    return Math.max(0, Math.min(1, done / total))
  }

  function priceUnitsPerToken(priceAtomic: bigint): string {
    return `${currencyDisplay.displayValue(priceAtomic)} ${currencyInfo.symbol}/${tokenInfo.symbol}`
  }

  async function refreshAuction() {
    const ai = unwrapOption(await actor.auction_info())
    auctionInfo = ai
  }

  async function refreshGrouped() {
    try {
      const precisionAtomic = BigInt(groupedPrecision)
      console.log(groupedPrecision, precisionAtomic)
      grouped = await actor.get_grouped_bids([precisionAtomic])
    } catch {}
  }

  async function refreshMine() {
    if (!authStore.identity.isAuthenticated) {
      myInfo = {
        token_amount: 0n,
        currency_amount: 0n,
        agreed_terms: false,
        timestamp: 0n,
        bound_addresses: []
      }
      myBids = []
      return
    }
    const infoRes = await actor.my_info()
    myInfo = unwrapResult(infoRes, 'failed to fetch my info')

    const bidsRes = await actor.my_bids()
    myBids = unwrapResult(bidsRes, 'failed to fetch my bids')
  }

  let isSigningIn = $state(false)
  function onSignWith() {
    if (isSigningIn) return

    isSigningIn = true
    const result = authStore.signIn()

    toastRun(async () => {
      await result
    }).finally(() => {
      isSigningIn = false
    })
  }

  const [bidAmountUnits, bidAmountErr] = $derived.by(() => {
    if (!bidAmount.trim()) return [0n, '']
    try {
      const amt = parseUnits(bidAmount, currencyInfo.decimals)
      if (auctionCfg) {
        if (amt < auctionCfg.min_amount)
          return [
            0n,
            `The lowest bid is ${currencyDisplay.displayValue(auctionCfg.min_amount)} ${currencyInfo.symbol}`
          ]
        if (amt > auctionCfg.max_amount)
          return [
            0n,
            `The highest bid is ${currencyDisplay.displayValue(auctionCfg.max_amount)} ${currencyInfo.symbol}`
          ]
        if (amt > myInfo.currency_amount)
          return [
            0n,
            `You only have ${currencyDisplay.displayValue(myInfo.currency_amount)} ${currencyInfo.symbol} available`
          ]
      }
      return [amt, '']
    } catch (e) {
      return [0n, 'Invalid bid amount']
    }
  })

  let isEstimating = $state(false)
  let bidMaxPriceLimit = $state(0n)
  function estimateMaxPrice() {
    toastRun(async () => {
      if (isEstimating || bidAmountUnits == 0n) return
      if (!stateInfo) throw new Error('state not ready')
      const [_priceAtomic, priceLimitAtomic] =
        await actor.estimate_max_price(bidAmountUnits)
      bidMaxPriceLimit = priceLimitAtomic
      bidMaxPrice = currencyDisplay.displayValue(
        priceLimitAtomic + priceLimitAtomic / 10n // add 10% buffer
      )
    }).finally(() => {
      isEstimating = false
    })
  }

  const [bidMaxPriceUnits, bidMaxPriceErr] = $derived.by(() => {
    if (!bidMaxPrice.trim()) return [0n, '']
    try {
      const amt = parseUnits(bidMaxPrice, currencyInfo.decimals)
      if (amt < bidMaxPriceLimit)
        return [
          0n,
          `The max price must be at least ${currencyDisplay.displayValue(
            bidMaxPriceLimit
          )} ${currencyInfo.symbol}`
        ]

      return [amt, '']
    } catch (e) {
      return [0n, 'Invalid bid amount']
    }
  })

  let isBidding = $state(false)
  const isBidable = $derived.by(() => {
    return (
      !isBidding &&
      bidAmountErr === '' &&
      bidMaxPriceErr === '' &&
      bidAmountUnits > 0n &&
      bidAmountUnits <= myInfo.currency_amount &&
      bidMaxPriceUnits > 0n &&
      (phase == 'prebid' || phase == 'bidding')
    )
  })

  function submitBid() {
    if (!stateInfo || !auctionCfg || !auctionInfo) {
      throw new Error('auction not ready')
    }
    if (!isBidable || isBidding) return
    isBidding = true
    toastRun(async () => {
      const res = await actor.submit_bid(bidAmountUnits, bidMaxPriceUnits)
      const bid = unwrapResult(res, 'submit bid failed')
      myBids.push(bid)
      triggerToast({ type: 'success', message: 'Bid has been submitted' })

      await refreshAuction()
      await refreshMine()
    }).finally(() => {
      bidAmount = ''
      bidMaxPrice = ''
      isBidding = false
    })
  }

  async function claimOne(id: bigint) {
    const res = await actor.claim(id)
    unwrapResult(res, 'claim failed')
    triggerToast({ type: 'success', message: 'Received/Refund completed' })
    await refreshMine()
  }

  async function claimAll() {
    const res = await actor.claim_all()
    unwrapResult(res, 'claim all failed')
    triggerToast({ type: 'success', message: 'Received/Refund completed' })
    await refreshMine()
  }

  function onBidAmountFocus() {
    bidMaxPrice = '' // ensure re-estimation
    if (bidAmount == '' && myInfo.currency_amount > 0n) {
      bidAmount = currencyDisplay.displayValue(myInfo.currency_amount)
    }
  }

  function onCCAModal() {
    showModal({
      title: 'Continuous Clearing Auction',
      component: CCAModal,
      size: 'xl'
    })
  }

  function onCanisterModal() {
    showModal({
      title: 'Auction Smart Contract',
      component: CanisterModal,
      size: 'xl',
      componentProps: {
        stateInfo
      }
    })
  }

  function onUserFundsModal() {
    showModal({
      title: 'My Funds',
      component: UserFundsModal,
      size: 'xl',
      componentProps: {
        stateInfo,
        auction: actor,
        myInfo
      }
    })
  }

  let timer: any

  onMount(() => {
    return toastRun(async (_signal, abortingQue) => {
      const id =
        page.params['id'] == 'latest'
          ? ''
          : Principal.fromText(page.params['id'] || '').toString()
      const auction = await listingActor.get_auction(id ? [{ Icp: id }] : [])
      isListed = auction[0] != null && 'Icp' in auction[0].id
      canister = id
        ? id
        : auction[0] && 'Icp' in auction[0].id
          ? auction[0].id.Icp
          : canister

      const sres = await actor.info()
      const s = unwrapResult(sres, 'failed to fetch auction state')
      stateInfo = s
      auctionCfg = unwrapOption(s.auction_config)
      tokenInfo = {
        name: s.token_name,
        symbol: s.token_symbol,
        decimals: s.token_decimals,
        fee: 0n,
        one: 10n ** BigInt(s.token_decimals),
        logo: s.token_logo_url,
        address: s.token
      }
      currencyInfo = {
        name: s.currency_name,
        symbol: s.currency_symbol,
        decimals: s.currency_decimals,
        fee: 0n,
        one: 10n ** BigInt(s.currency_decimals),
        logo: s.currency_logo_url,
        address: s.currency
      }

      await tick()
      if (auctionCfg) {
        floorPrice =
          (auctionCfg.required_currency_raised * tokenInfo.one) /
          auctionCfg.total_supply
      }

      await tick()
      groupedPrecision = (floorGroupedPrecision * 5n).toString()

      await refreshAuction()
      if (authStore.identity.isAuthenticated) {
        await refreshMine()
      } else {
        authStore.addEventListener(EventLogin, refreshMine)
        abortingQue.push(() => {
          authStore.removeEventListener(EventLogin, refreshMine)
        })
      }
    }).abort
  })

  onDestroy(() => {
    if (timer) clearInterval(timer)
  })
</script>

<div class="relative flex min-h-screen flex-col overflow-x-hidden">
  <!-- Global decorative elements -->
  <div
    class="pointer-events-none fixed inset-0 overflow-hidden"
    aria-hidden="true"
  >
    <div
      class="animate-float-slow absolute -top-40 -left-40 h-80 w-80 rounded-full bg-purple-500/10 blur-3xl"
    ></div>
    <div
      class="animate-float absolute -top-20 -right-20 h-96 w-96 rounded-full bg-amber-500/10 blur-3xl"
    ></div>
    <div
      class="animate-float-reverse absolute top-1/2 left-1/4 h-64 w-64 rounded-full bg-yellow-500/5 blur-3xl"
    ></div>
    <div
      class="animate-float absolute right-1/4 bottom-20 h-72 w-72 rounded-full bg-amber-400/5 blur-3xl"
    ></div>
    <div class="grid-pattern absolute inset-0"></div>
  </div>

  <Header backUrl="/_/launchpad" description={'Continuous Clearing Auction'} />

  {#if !isListed}
    <div class="text-md mx-auto mt-2 -mb-4 text-center text-red-600 md:-mb-8">
      This auction is not listed on TokenList.ing
    </div>
  {/if}
  <main
    class="relative z-10 mx-auto w-full max-w-6xl space-y-6 px-4 py-6 md:px-8 md:py-10"
  >
    {#if !stateInfo}
      <div
        class="glass-border flex items-center justify-center rounded-xl p-10"
      >
        <Spinner class="h-6 w-6" />
      </div>
    {:else}
      <!-- Top summary -->
      <section class="grid gap-4 lg:grid-cols-3">
        <div
          class="glass-border relative overflow-hidden rounded-xl p-4 md:p-6 lg:col-span-2"
        >
          <div
            class="pointer-events-none absolute inset-0 opacity-40"
            aria-hidden="true"
          >
            <div
              class="animate-pulse-glow absolute -top-10 -right-10 h-40 w-40 rounded-full bg-purple-500/10 blur-3xl"
            ></div>
            <div
              class="animate-pulse-glow absolute -bottom-10 -left-10 h-40 w-40 rounded-full bg-amber-500/10 blur-3xl"
            ></div>
          </div>

          <div class="relative space-y-4">
            <div class="flex flex-col gap-3 sm:items-start sm:justify-between">
              <div class="flex w-full flex-row items-center justify-between">
                <div class="flex items-center gap-2">
                  <span
                    class="text-muted text-xs font-semibold tracking-wide uppercase"
                    >Project</span
                  >
                  {#if stateInfo.url}
                    <a
                      class="border-border-subtle text-muted hover:border-foreground hover:text-foreground inline-flex items-center gap-1 rounded-full border px-2 py-1 text-xs font-semibold tracking-wide uppercase"
                      href={stateInfo.url}
                      target="_blank"
                      rel="noreferrer"
                    >
                      Website
                      <ArrowRightUpLine class="h-4 w-4" />
                    </a>
                  {/if}
                </div>
                <div class="flex items-center gap-2">
                  <button
                    class="border-border-subtle text-muted hover:border-foreground hover:text-foreground inline-flex items-center gap-1 rounded-full border px-2 py-1 text-xs font-semibold tracking-wide uppercase"
                    onclick={onCCAModal}
                    ><InformationLine class="h-4 w-4" />
                    CCA
                  </button>
                  <button
                    class="border-border-subtle text-muted hover:border-foreground hover:text-foreground inline-flex items-center gap-1 rounded-full border px-1 py-1 text-xs font-semibold tracking-wide uppercase"
                    onclick={onCanisterModal}
                    ><Settings4Line class="h-4 w-4" />
                  </button>
                </div>
              </div>
              <div class="text-xl font-bold">
                {stateInfo.name || 'Auction'}
                <span class="text-muted font-semibold"
                  >· {stateInfo.token_symbol}</span
                >
              </div>
              {#if detailText}
                <div class="space-y-2">
                  <div
                    class={`md-content w-full text-pretty wrap-break-word ${!isDetailExpanded && isDetailLong ? 'cca-desc-clamp' : ''}`}
                  >
                    {@html renderContent(detailText)}
                  </div>
                  {#if isDetailLong}
                    <button
                      class="inline-flex items-center gap-1 text-xs font-semibold tracking-wide text-indigo-500 uppercase hover:text-indigo-700"
                      onclick={() => (isDetailExpanded = !isDetailExpanded)}
                      type="button"
                    >
                      {isDetailExpanded ? 'Show less' : 'Show more'}
                    </button>
                  {/if}
                </div>
              {:else}
                <div class="md-content w-full text-pretty wrap-break-word"
                  >—</div
                >
              {/if}
            </div>

            <div class="grid gap-3 sm:grid-cols-3">
              <div
                class="border-border-subtle bg-surface group relative rounded-lg border p-3"
                aria-label="Phase"
              >
                <div
                  class="text-muted text-xs font-semibold tracking-wide uppercase"
                  >Phase</div
                >
                <div class="mt-1 text-lg font-bold">
                  {phase}
                </div>

                <!-- Hover tooltip: show full phase flow -->
                <div
                  class=" bg-card pointer-events-none absolute bottom-full left-0 z-20 mb-2 w-40 rounded-lg p-3 text-xs opacity-0 shadow transition-opacity group-hover:opacity-100"
                  role="tooltip"
                >
                  <div
                    class="text-muted mb-2 font-semibold tracking-wide uppercase"
                  >
                    All phases
                  </div>
                  <div class="space-y-1">
                    <div
                      class={phase === 'unconfigured'
                        ? 'text-foreground font-semibold'
                        : 'text-muted'}
                    >
                      unconfigured
                    </div>
                    <div
                      class={phase === 'configured'
                        ? 'text-foreground font-semibold'
                        : 'text-muted'}
                    >
                      configured
                    </div>
                    <div
                      class={phase === 'prebid'
                        ? 'text-foreground font-semibold'
                        : 'text-muted'}
                    >
                      prebid
                    </div>
                    <div
                      class={phase === 'bidding'
                        ? 'text-foreground font-semibold'
                        : 'text-muted'}
                    >
                      bidding
                    </div>
                    <div
                      class={phase === 'ending'
                        ? 'text-foreground font-semibold'
                        : 'text-muted'}
                    >
                      ending
                    </div>
                    <div
                      class={phase === 'ended'
                        ? 'text-foreground font-semibold'
                        : 'text-muted'}
                    >
                      ended
                    </div>
                  </div>
                </div>
              </div>

              <div
                class="border-border-subtle bg-surface rounded-lg border p-3"
              >
                <div
                  class="text-muted text-xs font-semibold tracking-wide uppercase"
                  >Bidders</div
                >
                <div class="mt-1 text-lg font-bold">
                  {stateInfo.total_bidders}
                </div>
              </div>
              <div
                class="border-border-subtle bg-surface rounded-lg border p-3"
              >
                <div
                  class="text-muted text-xs font-semibold tracking-wide uppercase"
                  >Graduated</div
                >
                <div class="mt-1 text-lg font-bold">
                  {auctionInfo?.is_graduated ? 'Yes' : 'No'}
                </div>
              </div>
            </div>

            {#if auctionCfg}
              <div class="space-y-2">
                <div class="flex items-center justify-between text-xs">
                  <div class="text-muted"
                    >{formatDatetime(auctionCfg.start_time)} → {formatDatetime(
                      auctionCfg.end_time
                    )}</div
                  >
                  <div class="text-muted"
                    >{Math.round(progress(auctionCfg) * 100)}%</div
                  >
                </div>
                <div class="bg-surface h-2 overflow-hidden rounded-full">
                  <div
                    class="h-2 rounded-full bg-linear-to-r from-purple-500 via-amber-500 to-yellow-400"
                    style={`width:${Math.round(progress(auctionCfg) * 100)}%`}
                  ></div>
                </div>
              </div>
            {/if}

            {#if stateInfo.finalize_output.length > 0}
              {@const txid = stateInfo.finalize_output[0]?.txid || ''}
              {@const txUrl = getTxUrl(stateInfo.chain, txid)}
              <div class="text-md font-semibold">
                {#if txUrl}
                  <a
                    class="hover:border-foreground hover:text-foreground inline-flex items-center gap-1"
                    href={txUrl}
                    target="_blank"
                    rel="noreferrer"
                  >
                    📝 Finalize Transaction: {txid}
                    <ArrowRightUpLine class="h-4 w-4" />
                  </a>
                {:else}
                  <span>📝 Finalize Transaction: {txid}</span>
                {/if}
              </div>
            {:else if stateInfo.restricted_countries.length > 0}
              <div class="text-muted text-xs">
                ⚠️ Excluded Persons:{' '}
                {stateInfo.restricted_countries
                  .map((p) => pruneAddress(p, false))
                  .join(', ')}
              </div>
            {/if}
          </div>
        </div>

        <div class="glass-border rounded-xl p-4 md:p-6">
          <div class="space-y-4">
            <div
              class="text-muted text-xs font-semibold tracking-wide uppercase"
              >Targets</div
            >
            {#if auctionCfg}
              {@const finalizeKind = getFinalizeKind()}
              <div class="space-y-2">
                <div class="flex items-center justify-between text-sm">
                  <div class="text-muted">Currency</div>
                  <a
                    class="hover:border-foreground hover:text-foreground inline-flex items-center gap-1 font-semibold"
                    href={getTokenUrl(
                      stateInfo.chain,
                      currencyDisplay.token.address
                    )}
                    target="_blank"
                    rel="noreferrer"
                  >
                    {currencyInfo.symbol}
                    {pruneAddress(currencyDisplay.token.address, false)}
                    <ArrowRightUpLine class="h-4 w-4" />
                  </a>
                </div>
                <div class="flex items-center justify-between text-sm">
                  <div class="text-muted">Token</div>
                  <a
                    class="hover:border-foreground hover:text-foreground inline-flex items-center gap-1 font-semibold"
                    href={getTokenUrl(
                      stateInfo.chain,
                      tokenDisplay.token.address
                    )}
                    target="_blank"
                    rel="noreferrer"
                  >
                    {tokenInfo.symbol}
                    {pruneAddress(tokenDisplay.token.address, false)}
                    <ArrowRightUpLine class="h-4 w-4" />
                  </a>
                </div>
                <div class="flex items-center justify-between text-sm">
                  <div class="text-muted">Graduation Threshold</div>
                  <div class="font-semibold">
                    {currencyDisplay.displayValue(
                      auctionCfg.required_currency_raised
                    )}
                    {currencyDisplay.token.symbol}
                  </div>
                </div>
                <div class="flex items-center justify-between text-sm">
                  <div class="text-muted">Auction Supply</div>
                  <div class="font-semibold">
                    {tokenDisplay.displayValue(auctionCfg.total_supply)}
                    {tokenDisplay.token.symbol}
                  </div>
                </div>
                <div class="flex items-center justify-between text-sm">
                  <div class="text-muted">Liquidity Supply</div>
                  <div class="font-semibold">
                    {tokenDisplay.displayValue(
                      auctionCfg.liquidity_pool_amount
                    )}
                    {tokenDisplay.token.symbol}
                  </div>
                </div>
                <div class="flex items-center justify-between text-sm">
                  <div class="text-muted">Min/Max Bid</div>
                  <div class="font-semibold">
                    {currencyDisplay.displayValue(
                      auctionCfg.min_amount
                    )}–{currencyDisplay.displayValue(auctionCfg.max_amount)}
                    {currencyDisplay.token.symbol}
                  </div>
                </div>
                <div class="flex items-center justify-between text-sm">
                  <div class="text-muted">Finalize</div>
                  <div class="font-semibold">
                    {#if finalizeKind[1]}
                      <a
                        class=" inline-flex items-center gap-1 text-indigo-500 hover:text-indigo-700"
                        href={finalizeKind[1]}
                        target="_blank"
                        rel="noreferrer"
                      >
                        {finalizeKind[0]}
                        <ArrowRightUpLine class="h-4 w-4" />
                      </a>
                    {:else}
                      <span>{finalizeKind[0]}</span>
                    {/if}
                  </div>
                </div>
              </div>
            {:else}
              <div class="text-muted text-sm">Auction unconfigured</div>
            {/if}
          </div>
        </div>
      </section>

      <!-- Live stats + bidding -->
      <section class="grid gap-4 lg:grid-cols-2">
        <div class="glass-border rounded-xl p-4 md:p-6">
          <div class="flex items-center justify-between">
            <div class="space-y-1">
              <div
                class="text-muted text-xs font-semibold tracking-wide uppercase"
                >Live Stats</div
              >
              <div class="text-lg font-bold">Market Clearing</div>
            </div>
          </div>

          <div class="mt-4 grid gap-3 sm:grid-cols-2">
            <div class="border-border-subtle bg-surface rounded-lg border p-3">
              <div class="text-muted text-xs">Cumulative Raised</div>
              <div class="mt-1 text-base font-bold">
                {auctionInfo
                  ? `${currencyDisplay.displayValue(auctionInfo.cumulative_demand_raised)} ${currencyInfo.symbol}`
                  : '—'}
              </div>
            </div>
            <div class="border-border-subtle bg-surface rounded-lg border p-3">
              <div class="text-muted text-xs">Supply Released</div>
              <div class="mt-1 text-base font-bold">
                {auctionInfo
                  ? `${tokenDisplay.displayValue(auctionInfo.cumulative_supply_released)} ${tokenInfo.symbol}`
                  : '—'}
              </div>
            </div>
            <div class="border-border-subtle bg-surface rounded-lg border p-3">
              <div class="text-muted text-xs">Clearing Price</div>
              <div class="mt-1 text-base font-bold">
                {auctionInfo
                  ? priceUnitsPerToken(auctionInfo.clearing_price)
                  : priceUnitsPerToken(floorPrice)}
              </div>
            </div>
            <div class="border-border-subtle bg-surface rounded-lg border p-3">
              <div class="text-muted text-xs">Bids</div>
              <div class="mt-1 text-base font-bold">
                {auctionInfo ? auctionInfo.bids_count : '0'}
              </div>
            </div>
          </div>

          <div class="mt-4">
            <div class="flex items-center justify-between">
              <div
                class="text-muted text-xs font-semibold tracking-wide uppercase"
                >Demand Distribution</div
              >
              <div class="flex items-center gap-2">
                <label class="text-muted text-xs" for="groupedPrecision"
                  >Bucket</label
                >
                <select
                  id="groupedPrecision"
                  class="border-border-subtle bg-card w-16 rounded-md border px-1 py-1 text-xs"
                  bind:value={groupedPrecision}
                  oninput={() => toastRun(refreshGrouped)}
                >
                  <option value={floorGroupedPrecision}
                    >{currencyDisplay.displayValue(
                      floorGroupedPrecision
                    )}</option
                  >
                  <option value={floorGroupedPrecision * 5n}
                    >{currencyDisplay.displayValue(
                      floorGroupedPrecision * 5n
                    )}</option
                  >
                  <option value={floorGroupedPrecision * 10n}
                    >{currencyDisplay.displayValue(
                      floorGroupedPrecision * 10n
                    )}</option
                  >
                  <option value={floorGroupedPrecision * 50n}
                    >{currencyDisplay.displayValue(
                      floorGroupedPrecision * 50n
                    )}</option
                  >
                  <option value={floorGroupedPrecision * 100n}
                    >{currencyDisplay.displayValue(
                      floorGroupedPrecision * 100n
                    )}</option
                  >
                </select>
                <span class="text-muted text-xs"
                  >{currencyInfo.symbol}/{tokenInfo.symbol}</span
                >
              </div>
            </div>

            {#if grouped.length === 0}
              <div class="text-muted mt-3 text-sm"
                >No visualized data (no active bids or overly detailed
                information)</div
              >
            {:else}
              {@const maxBucket = grouped.reduce(
                (m, x) => (x[1] > m ? x[1] : m),
                0n
              )}
              <div class="mt-3 space-y-2">
                {#each grouped as [p, a]}
                  <div class="flex items-center gap-3">
                    <div class="text-muted w-28 text-xs sm:w-32">
                      ≤ {currencyDisplay.displayValue(p)}
                    </div>
                    <div
                      class="bg-surface h-2 flex-1 overflow-hidden rounded-full"
                    >
                      <div
                        class="h-2 rounded-full bg-linear-to-r from-purple-500 via-amber-500 to-yellow-400"
                        style={`width:${maxBucket === 0n ? 0 : Number((a * 100n) / maxBucket)}%`}
                      ></div>
                    </div>
                    <div class="w-32 text-right text-xs font-semibold">
                      {currencyDisplay.displayValue(a)}
                      {currencyInfo.symbol}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <div class="glass-border rounded-xl p-4 md:p-6">
          <div class="space-y-1">
            <div
              class="text-muted text-xs font-semibold tracking-wide uppercase"
              >Bid</div
            >
            <div class="text-lg font-bold">Place a Bid</div>
          </div>

          {#if !auctionInfo || !auctionCfg}
            <div class="text-muted mt-4 text-sm"
              >The auction has not been set up yet</div
            >
          {:else}
            <div class="mt-4 grid gap-3">
              <div>
                <label class="text-muted text-xs" for="bidAmount">
                  <span class="font-semibold tracking-wide uppercase"
                    >Amount</span
                  >
                  <span class=""
                    >(Balance: {currencyDisplay.displayValue(
                      myInfo.currency_amount
                    )}
                    {currencyInfo.symbol})</span
                  >
                  <button
                    class="text-sm text-indigo-500 hover:text-indigo-700 disabled:cursor-not-allowed"
                    onclick={onUserFundsModal}
                    disabled={!stateInfo || !authStore.identity.isAuthenticated}
                  >
                    Deposit
                  </button>
                </label>

                <input
                  id="bidAmount"
                  class="border-border-subtle bg-card mt-1 w-full rounded-lg border px-3 py-2 text-sm"
                  placeholder={`e.g. ${currencyDisplay.displayValue(auctionCfg.min_amount)} ${currencyInfo.symbol}`}
                  bind:value={bidAmount}
                  onfocus={onBidAmountFocus}
                  inputmode="decimal"
                />
              </div>

              <div>
                <label class="text-muted text-xs" for="bidMaxPrice">
                  <span class="font-semibold tracking-wide uppercase"
                    >Max Price</span
                  >
                  <span class="text-muted text-xs">
                    {`(The maximum ${currencyInfo.symbol} price per 1 ${tokenInfo.symbol})`}</span
                  >
                </label>

                <input
                  id="bidMaxPrice"
                  class="border-border-subtle bg-card mt-1 w-full rounded-lg border px-3 py-2 text-sm"
                  placeholder={`e.g. ${currencyDisplay.displayValue(auctionInfo ? auctionInfo.clearing_price * 2n : floorPrice)} ${currencyInfo.symbol}/${tokenInfo.symbol}`}
                  bind:value={bidMaxPrice}
                  onfocus={estimateMaxPrice}
                  inputmode="decimal"
                />
              </div>

              <div class="flex flex-wrap items-center gap-2">
                <Button
                  class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
                  onclick={estimateMaxPrice}
                  isLoading={isEstimating}
                  disabled={bidAmountUnits == 0n}
                >
                  Estimate
                </Button>

                {#if authStore.identity.isAuthenticated}
                  <Button
                    class="bg-foreground text-background flex-1 rounded-full px-4 py-2 text-xs font-semibold tracking-wide uppercase hover:opacity-90 disabled:opacity-50"
                    onclick={submitBid}
                    isLoading={isBidding}
                    disabled={!isBidable}
                  >
                    Submit Bid
                  </Button>
                {:else}
                  <Button
                    class="bg-foreground text-background flex-1 rounded-full px-4 py-2 text-xs font-semibold tracking-wide uppercase hover:opacity-90 disabled:opacity-50"
                    onclick={onSignWith}
                    isLoading={isSigningIn}
                  >
                    Sign in
                  </Button>
                {/if}
              </div>

              <div
                class="border-border-subtle bg-surface rounded-lg border p-2 text-xs"
              >
                {#if bidAmountErr || bidMaxPriceErr}
                  <p class="text-red-500">
                    {bidAmountErr || bidMaxPriceErr}
                  </p>
                {:else}
                  <p class="">
                    Bids are streamed linearly over the remaining time. Early
                    entry maximizes capital deployment. <b
                      >Note that bids cannot be manually cancelled</b
                    >, but if the clearing price exceeds your max price, you
                    will be automatically <b>outbid</b> and unspent funds are refundable.
                  </p>
                {/if}
              </div>
            </div>
          {/if}
        </div>
      </section>

      <!-- My activity -->
      <section class="glass-border rounded-xl p-4 md:p-6">
        <div
          class="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between"
        >
          <div>
            <div
              class="text-muted text-xs font-semibold tracking-wide uppercase"
              >Account</div
            >
            <div class="text-lg font-bold">My Activity</div>
          </div>
          <div class="flex items-center gap-2">
            <button
              class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
              onclick={onUserFundsModal}
              disabled={!stateInfo || !authStore.identity.isAuthenticated}
            >
              My Funds
            </button>
            <Button
              class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
              onclick={() => toastRun(claimAll, 'claim all failed')}
              disabled={!authStore.identity.isAuthenticated ||
                myBids.length === 0}
            >
              Claim All
            </Button>
          </div>
        </div>

        <div class="mt-6">
          <div
            class="text-muted mb-2 text-xs font-semibold tracking-wide uppercase"
            >My Bids</div
          >
          {#if !auctionInfo || myBids.length === 0}
            <div class="text-muted text-sm">No bids yet.</div>
          {:else}
            <div class="border-border-subtle overflow-hidden rounded-xl border">
              <div
                class="bg-surface grid grid-cols-12 gap-2 px-3 py-2 text-xs font-semibold"
              >
                <div class="col-span-2">ID</div>
                <div class="col-span-3">Amount</div>
                <div class="col-span-3">Max Price</div>
                <div class="col-span-2">Status</div>
                <div class="col-span-2 text-right">Action</div>
              </div>
              {#each myBids as b (b.id)}
                {@const isOutbid = b.outbid_time.length === 1}
                {@const isClaimed = b.claim_time > 0n}
                {@const isClaimable =
                  !isClaimed &&
                  ((isOutbid && auctionInfo.is_graduated) || phase == 'ended')}
                <div
                  class="border-border-subtle grid grid-cols-12 gap-2 border-t px-3 py-2 text-xs"
                >
                  <div class="col-span-2 font-semibold">{b.id.toString()}</div>
                  <div class="col-span-3">
                    {currencyDisplay.displayValue(b.amount)}
                    {currencyInfo.symbol}
                  </div>
                  <div class="col-span-3">{priceUnitsPerToken(b.max_price)}</div
                  >
                  <div class="col-span-2">
                    {#if isClaimed}
                      <span class="text-muted">claimed</span>
                    {:else if isOutbid}
                      <span class="text-muted">outbid</span>
                    {:else}
                      <span class="text-muted">active</span>
                    {/if}
                  </div>
                  <div class="col-span-2 text-right">
                    <Button
                      class="border-border-subtle text-muted hover:border-foreground hover:text-foreground disabled:text-muted-foreground rounded-full border px-2 py-1 text-xs font-semibold {isClaimed
                        ? 'invisible'
                        : ''}"
                      onclick={() =>
                        toastRun(() => claimOne(b.id), 'claim failed')}
                      disabled={!isClaimable}
                    >
                      Claim
                    </Button>
                  </div>
                </div>
                {#if b.tokens_filled > 0n || b.refund > 0n}
                  <div
                    class="border-border-subtle bg-surface grid grid-cols-12 gap-2 border-t px-3 py-2 text-xs"
                  >
                    <div class="text-muted col-span-12">
                      Filled: <span class="text-foreground font-semibold"
                        >{tokenDisplay.displayValue(b.tokens_filled)}
                        {tokenInfo.symbol}</span
                      >
                      · Refund:
                      <span class="text-foreground font-semibold"
                        >{currencyDisplay.displayValue(b.refund)}
                        {currencyInfo.symbol}</span
                      >
                    </div>
                  </div>
                {/if}
              {/each}
            </div>
          {/if}
        </div>
      </section>
    {/if}
  </main>
</div>

<style>
  .cca-desc-clamp {
    overflow: hidden;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    line-clamp: 3;
    -webkit-line-clamp: 3;
  }
</style>
