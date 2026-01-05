<script lang="ts">
  import { page } from '$app/state'
  import type {
    AuctionConfig,
    AuctionInfo,
    AuctionSnapshot,
    BidInfo,
    StateInfo,
    UserInfo
  } from '$declarations/ic_auction/ic_auction.did'
  import { icAuctionActor } from '$lib/canisters/icAuction'
  import { tokenListingActor } from '$lib/canisters/tokenListing'
  import CanisterModal from '$lib/components/CanisterModal.svelte'
  import CCAModal from '$lib/components/CCAModal.svelte'
  import Header from '$lib/components/Header.svelte'
  import PriceDiscoveryChart from '$lib/components/PriceDiscoveryChart.svelte'
  import UserFundsModal from '$lib/components/UserFundsModal.svelte'
  import { TOKEN_LISTING } from '$lib/constants'
  import ArrowDownSLine from '$lib/icons/arrow-down-s-line.svelte'
  import ArrowRightUpLine from '$lib/icons/arrow-right-up-line.svelte'
  import CheckLine from '$lib/icons/check-line.svelte'
  import InformationLine from '$lib/icons/information-line.svelte'
  import Settings4Line from '$lib/icons/settings-4-line.svelte'
  import { authStore, EventLogin } from '$lib/stores/auth.svelte'
  import { showModal } from '$lib/stores/modal.svelte'
  import { toastRun, triggerToast } from '$lib/stores/toast.svelte'
  import { unwrapOption, unwrapResult } from '$lib/types/result'
  import Button from '$lib/ui/Button.svelte'
  import Dropdown from '$lib/ui/Dropdown.svelte'
  import Spinner from '$lib/ui/Spinner.svelte'
  import Tooltip from '$lib/ui/Tooltip.svelte'
  import {
    getAccountUrl,
    getSwapUrl,
    getTokenUrl,
    getTxUrl
  } from '$lib/utils/chain'
  import {
    formatDatetime,
    parseUnits,
    pruneAddress,
    sleep
  } from '$lib/utils/helper'
  import {
    PANDAToken,
    TokenDisplay,
    USDCToken,
    type TokenInfo
  } from '$lib/utils/token'
  import { isActive } from '$lib/utils/window'
  import MarkdownContent from '$src/lib/ui/MarkdownContent.svelte'
  import { Principal } from '@dfinity/principal'
  import { onDestroy, onMount, tick } from 'svelte'

  const listingActor = tokenListingActor(TOKEN_LISTING)

  let canister = $state('kfzgd-diaaa-aaaap-an56q-cai')
  const actor = $derived(icAuctionActor(canister))

  let nowMs = $state(Date.now())
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
  let snapshots = $state<Array<AuctionSnapshot>>([])
  let tokenInfo = $state<TokenInfo>(PANDAToken)
  let currencyInfo = $state<TokenInfo>(USDCToken)

  const tokenDisplay = $derived.by(() => new TokenDisplay(tokenInfo, 0n))
  const currencyDisplay = $derived.by(() => new TokenDisplay(currencyInfo, 0n))

  let myBids = $state<BidInfo[]>([])

  // Inputs
  let floorPrice = $state(USDCToken.one)
  let bidAmount = $state('')
  let bidMaxPrice = $state('')

  const detailText = $derived.by(() =>
    (stateInfo?.detail || stateInfo?.description || '').trim()
  )

  let floorGroupedPrecision = $derived.by(() => {
    const val = Math.max(floorPrice.toString().length - 1, 1)
    return 10n ** BigInt(val)
  })
  const phase = $derived.by(() => {
    if (!auctionCfg) return 'unconfigured' as const
    if (!auctionInfo) return 'configured' as const

    const n = BigInt(Math.max(Date.now(), nowMs))
    if (n < auctionCfg.start_time) return 'pre-bidding' as const
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
    const n = Math.max(Date.now(), nowMs)
    const start = Number(cfg.start_time)
    const end = Number(cfg.end_time)
    if (end <= start) return 0
    if (n <= start) return 0
    if (n >= end) return 1
    const done = n - start
    const total = end - start
    return Math.max(0, Math.min(1, done / total))
  }

  function priceUnitsPerToken(priceAtomic: bigint): string {
    return `${currencyDisplay.displayValue(priceAtomic)} ${currencyInfo.symbol}/${tokenInfo.symbol}`
  }

  let openPriceBucket = $state(false)
  let isLoadingBuckets = $state(false)
  let groupedPrecision = $state(0n)
  const handlePriceBucket = (precision: bigint) => {
    openPriceBucket = false
    if (isLoadingBuckets) return
    groupedPrecision = precision
    toastRun(refreshGrouped)
  }

  async function refreshGrouped() {
    try {
      isLoadingBuckets = true
      grouped = await actor.get_grouped_bids([groupedPrecision])
    } finally {
      isLoadingBuckets = false
    }
  }

  let fromTimestamp = $state<bigint>(0n)
  let isRefreshingSnapshots = $state(false)

  function mergeSnapshots(next: AuctionSnapshot[]) {
    if (next.length === 0) return

    // Ensure chronological order from API.
    next.sort((a, b) => (a.t < b.t ? -1 : a.t > b.t ? 1 : 0))

    // De-duplicate/merge by timestamp.
    const indexByT = new Map<bigint, number>()
    for (let i = 0; i < snapshots.length; i++) {
      const s = snapshots[i]
      if (s) indexByT.set(s.t, i)
    }

    for (const s of next) {
      const idx = indexByT.get(s.t)
      if (idx === undefined) {
        indexByT.set(s.t, snapshots.length)
        snapshots.push(s)
      } else {
        snapshots[idx] = s
      }
    }

    // Keep local list sorted (cheap, snapshots is bounded).
    snapshots.sort((a, b) => (a.t < b.t ? -1 : a.t > b.t ? 1 : 0))
  }

  async function refreshSnapshot() {
    if (isRefreshingSnapshots) return
    isRefreshingSnapshots = true
    try {
      while (true) {
        const start = fromTimestamp
        const res = await actor.get_snapshots(start, 100n)
        if (res.length === 0) break

        // Filter out anything older than the requested start (defensive).
        const filtered = res.filter((s) => s.t >= start)
        mergeSnapshots(filtered)

        // Advance cursor by the maximum timestamp actually observed.
        let maxT = start
        for (const s of filtered) if (s.t > maxT) maxT = s.t
        fromTimestamp = maxT + 1n // next timestamp

        if (res.length !== 100) break
      }
    } catch {
    } finally {
      isRefreshingSnapshots = false
    }
  }

  async function refreshAuction() {
    const ai = await actor.auction_info()
    auctionInfo = unwrapOption(ai)

    await refreshGrouped()
    await refreshSnapshot()
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
            amt,
            `Insufficient balance: ${currencyDisplay.displayValue(myInfo.currency_amount)} ${currencyInfo.symbol} available`
          ]
      }
      return [amt, '']
    } catch (e) {
      return [0n, 'Invalid bid amount']
    }
  })

  let isEstimating = $state(false)
  let bidMaxPriceLimit = $state(0n)
  let estimateClearingPrice = $state('')
  function estimateMaxPrice() {
    toastRun(async () => {
      if (isEstimating || bidAmountUnits == 0n) return
      if (!stateInfo) throw new Error('state not ready')
      const [priceAtomic, priceLimitAtomic] =
        await actor.estimate_max_price(bidAmountUnits)
      bidMaxPriceLimit = priceLimitAtomic
      estimateClearingPrice = currencyDisplay.displayValue(priceAtomic)
      bidMaxPrice = currencyDisplay.displayValue(
        priceLimitAtomic * 2n // add 100% buffer
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
      (phase == 'pre-bidding' || phase == 'bidding')
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

      await sleep(2000)
      await refreshAuction()
      await refreshMine()
    }).finally(() => {
      bidAmount = ''
      bidMaxPrice = ''
      estimateClearingPrice = ''
      isBidding = false
    })
  }

  async function claimOne(id: bigint) {
    const res = await actor.claim(id)
    unwrapResult(res, 'claim failed')
    triggerToast({ type: 'success', message: 'Received/Refund completed' })
    await sleep(2000)
    await refreshMine()
  }

  async function claimAll() {
    const res = await actor.claim_all()
    unwrapResult(res, 'claim all failed')
    triggerToast({ type: 'success', message: 'Received/Refund completed' })
    await sleep(2000)
    await refreshMine()
  }

  function onBidAmountFocus() {
    bidMaxPrice = '' // ensure re-estimation
    estimateClearingPrice = ''
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
        myInfo,
        onMyInfoChange: (next: UserInfo) => (myInfo = next)
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
      groupedPrecision = floorGroupedPrecision * 5n

      await refreshAuction()
      if (authStore.identity.isAuthenticated) {
        await refreshMine()
      } else {
        authStore.addEventListener(EventLogin, refreshMine)
        abortingQue.push(() => {
          authStore.removeEventListener(EventLogin, refreshMine)
        })
      }

      timer = setInterval(() => {
        nowMs = Date.now()
        if (
          isActive() &&
          (phase == 'pre-bidding' || phase == 'bidding' || phase == 'ending')
        ) {
          refreshAuction().catch((err) => {
            console.error('failed to refresh auction:', err)
          })
        } else if (phase == 'ended' && timer) {
          // Stop timer after auction ended
          clearInterval(timer)
          timer = null
        }
      }, 10000)
      abortingQue.push(() => {
        if (timer) clearInterval(timer)
        timer = null
      })
    }).abort
  })

  onDestroy(() => {
    if (timer) clearInterval(timer)
    timer = null
  })
</script>

{#snippet priceBucketTrigger()}
  <div class="flex min-w-0 items-center gap-1 text-sm">
    <span class="font-medium"
      >{currencyDisplay.displayValue(groupedPrecision)}</span
    >
    <ArrowDownSLine class="size-4" />
  </div>
{/snippet}

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
    class="relative z-10 mx-auto w-full max-w-6xl space-y-8 px-4 py-8 md:px-8 md:py-12"
  >
    {#if !stateInfo}
      <div
        class="glass-border flex h-64 items-center justify-center rounded-2xl"
      >
        <Spinner class="h-8 w-8 text-indigo-500" />
      </div>
    {:else}
      <!-- Project Hero Section -->
      <section class="grid gap-6 lg:grid-cols-3">
        <div
          class="glass-border relative overflow-hidden rounded-3xl p-6 md:p-8 lg:col-span-2"
        >
          <div
            class="pointer-events-none absolute inset-0 opacity-40"
            aria-hidden="true"
          >
            <div
              class="animate-pulse-glow absolute -top-20 -right-20 h-64 w-64 rounded-full bg-indigo-500/10 blur-[100px]"
            ></div>
            <div
              class="animate-pulse-glow absolute -bottom-20 -left-20 h-64 w-64 rounded-full bg-amber-500/10 blur-[100px]"
            ></div>
          </div>

          <div class="relative space-y-6">
            <div class="flex flex-col gap-4">
              <div class="flex items-center justify-between">
                <div class="group flex min-w-0 items-center gap-3">
                  <div
                    class="bg-surface ring-border-subtle h-14 w-14 shrink-0 overflow-hidden rounded-2xl p-1 shadow-inner ring-1"
                  >
                    {#if stateInfo.token_logo_url}
                      <img
                        class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-110"
                        src={stateInfo.token_logo_url}
                        alt={stateInfo.token_symbol}
                      />
                    {/if}
                  </div>
                  <div class="min-w-0">
                    <div
                      class="text-[10px] font-bold tracking-[0.2em] text-indigo-500 uppercase sm:text-xs"
                    >
                      Project
                    </div>
                    <h1
                      class="truncate font-serif text-2xl font-bold tracking-tight sm:text-3xl"
                    >
                      {stateInfo.name || 'Auction'}
                      <span class="text-muted font-medium"
                        >· {stateInfo.token_symbol}</span
                      >
                    </h1>
                  </div>
                </div>

                <div class="flex shrink-0 items-center gap-2">
                  {#if stateInfo.url}
                    <a
                      class="glass-border bg-surface/50 hover:bg-surface text-muted hover:text-foreground flex h-10 w-10 items-center justify-center rounded-xl transition-all"
                      href={stateInfo.url}
                      target="_blank"
                      rel="noreferrer"
                      title="Website"
                    >
                      <ArrowRightUpLine class="h-5 w-5" />
                    </a>
                  {/if}
                  <button
                    class="glass-border bg-surface/50 hover:bg-surface text-muted hover:text-foreground flex h-10 w-10 items-center justify-center rounded-xl transition-all"
                    onclick={onCCAModal}
                    title="CCA Info"
                  >
                    <InformationLine class="h-5 w-5" />
                  </button>
                  <button
                    class="glass-border bg-surface/50 hover:bg-surface text-muted hover:text-foreground flex h-10 w-10 items-center justify-center rounded-xl transition-all"
                    onclick={onCanisterModal}
                    title="Contract Settings"
                  >
                    <Settings4Line class="h-5 w-5" />
                  </button>
                </div>
              </div>

              <div class="max-w-2xl">
                {#key detailText}
                  <MarkdownContent content={detailText} />
                {/key}
              </div>
            </div>

            <div class="grid gap-4 sm:grid-cols-3">
              <Tooltip
                containerClass="w-full"
                contentClass="bg-card w-64 rounded-2xl p-5 shadow-2xl border border-white/10 backdrop-blur-xl"
              >
                {#snippet trigger()}
                  <div
                    class="glass-border bg-surface/40 group hover:bg-surface/60 relative rounded-2xl p-4 transition-all"
                    aria-label="Phase"
                  >
                    <div class="flex items-center justify-between">
                      <div
                        class="text-muted text-xs font-bold tracking-widest uppercase"
                        >Phase</div
                      >
                      <InformationLine
                        class="text-muted h-3.5 w-3.5 transition-colors group-hover:text-indigo-500"
                      />
                    </div>
                    <div class="mt-1 text-lg font-bold capitalize">
                      {phase.replace('-', ' ')}
                    </div>
                  </div>
                {/snippet}
                <div
                  class="text-muted mb-5 text-xs font-bold tracking-[0.2em] uppercase"
                >
                  Auction Lifecycle
                </div>
                <div class="space-y-0">
                  {#each ['configured', 'pre-bidding', 'bidding', 'ending', 'ended'] as p, i}
                    {@const isCurrent = phase === p}
                    {@const isPast =
                      [
                        'configured',
                        'pre-bidding',
                        'bidding',
                        'ending',
                        'ended'
                      ].indexOf(phase) > i}
                    <div class="relative flex items-start gap-4 pb-6 last:pb-0">
                      {#if i !== 4}
                        <div
                          class="absolute top-5 left-[9px] h-full w-0.5 {isPast
                            ? 'bg-indigo-500/50'
                            : 'bg-border-subtle'}"
                        ></div>
                      {/if}
                      <div
                        class="relative z-10 flex h-5 w-5 items-center justify-center rounded-full border-2 transition-all duration-500 {isCurrent
                          ? 'border-indigo-500 bg-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.5)]'
                          : isPast
                            ? 'border-indigo-500 bg-indigo-500'
                            : 'border-muted bg-surface'}"
                      >
                        {#if isPast}
                          <CheckLine class="h-3 w-3 text-white" />
                        {:else if isCurrent}
                          <div
                            class="absolute inset-0 animate-ping rounded-full bg-indigo-500/40"
                          ></div>
                          <div class="h-1.5 w-1.5 rounded-full bg-white"></div>
                        {/if}
                      </div>
                      <div class="flex min-h-5 flex-col justify-center">
                        <span
                          class="text-sm font-bold capitalize transition-colors {isCurrent
                            ? 'text-foreground'
                            : isPast
                              ? 'text-muted'
                              : 'text-muted/40'}"
                        >
                          {p.replace('-', ' ')}
                        </span>
                        {#if isCurrent}
                          <span
                            class="text-[9px] font-black tracking-widest text-indigo-500 uppercase"
                            >Current Phase</span
                          >
                        {/if}
                      </div>
                    </div>
                  {/each}
                </div>
              </Tooltip>

              <div class="glass-border bg-surface/40 rounded-2xl p-4">
                <div
                  class="text-muted text-xs font-bold tracking-widest uppercase"
                  >Bidders / Bids</div
                >
                <div class="mt-1 truncate text-lg font-bold">
                  {auctionInfo?.total_bidders || stateInfo.total_bidders}
                  <span class="text-muted text-sm font-medium">/</span>
                  {auctionInfo?.bids_count || 0}
                </div>
              </div>
              <div class="glass-border bg-surface/40 rounded-2xl p-4">
                <div
                  class="text-muted text-xs font-bold tracking-widest uppercase"
                  >Graduated</div
                >
                <div class="mt-1 text-lg font-bold">
                  {#if auctionInfo?.is_graduated}
                    <span class="text-emerald-500">Yes</span>
                  {:else}
                    <span class="text-muted">No</span>
                  {/if}
                </div>
              </div>
            </div>

            {#if stateInfo.finalize_output.length > 0}
              {@const txid = stateInfo.finalize_output[0]?.txid || ''}
              {@const txUrl = getTxUrl(stateInfo.chain, txid)}
              <div
                class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-3"
              >
                <div class="flex items-center gap-2 text-sm font-bold">
                  <span class="text-emerald-500">Finalized</span>
                  {#if txUrl}
                    <a
                      class="text-muted hover:text-foreground flex items-center gap-1 transition-colors"
                      href={txUrl}
                      target="_blank"
                      rel="noreferrer"
                    >
                      TX: {pruneAddress(txid, true)}
                      <ArrowRightUpLine class="h-3.5 w-3.5" />
                    </a>
                  {:else}
                    <span class="text-muted"
                      >TX: {pruneAddress(txid, true)}</span
                    >
                  {/if}
                </div>
              </div>
            {:else if stateInfo.restricted_countries.length > 0}
              <div
                class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-3"
              >
                <p class="text-muted text-xs leading-relaxed">
                  <strong class="text-amber-500 uppercase"
                    >Excluded Persons:</strong
                  >
                  {stateInfo.restricted_countries
                    .map((p) => pruneAddress(p, false))
                    .join(', ')}
                </p>
              </div>
            {/if}
          </div>
        </div>

        <!-- Targets Card -->
        <div class="glass-border bg-surface/30 flex flex-col rounded-3xl p-6">
          <div
            class="text-muted mb-6 text-xs font-bold tracking-widest uppercase"
            >Auction Targets</div
          >
          {#if auctionCfg}
            {@const finalizeKind = getFinalizeKind()}
            <div class="flex-1 space-y-4">
              <div class="flex items-center justify-between">
                <div class="text-muted text-xs font-medium">Currency</div>
                <a
                  class="flex items-center gap-1.5 text-sm font-bold transition-colors hover:text-indigo-500"
                  href={getTokenUrl(
                    stateInfo.chain,
                    currencyDisplay.token.address
                  )}
                  target="_blank"
                  rel="noreferrer"
                >
                  {currencyInfo.symbol}
                  <ArrowRightUpLine class="text-muted h-3.5 w-3.5" />
                </a>
              </div>
              <div class="flex items-center justify-between">
                <div class="text-muted text-xs font-medium">Token</div>
                <a
                  class="flex items-center gap-1.5 text-sm font-bold transition-colors hover:text-indigo-500"
                  href={getTokenUrl(
                    stateInfo.chain,
                    tokenDisplay.token.address
                  )}
                  target="_blank"
                  rel="noreferrer"
                >
                  {tokenInfo.symbol}
                  <ArrowRightUpLine class="text-muted h-3.5 w-3.5" />
                </a>
              </div>
              <div class="border-border-subtle border-t pt-4">
                <div class="flex items-center justify-between">
                  <div class="text-muted text-xs font-medium">Graduation</div>
                  <div class="text-sm font-bold">
                    {currencyDisplay.displayValue(
                      auctionCfg.required_currency_raised
                    )}
                    <span class="text-muted font-medium"
                      >{currencyDisplay.token.symbol}</span
                    >
                  </div>
                </div>
              </div>
              <div class="flex items-center justify-between">
                <div class="text-muted text-xs font-medium">Supply</div>
                <div class="text-sm font-bold">
                  {tokenDisplay.displayValue(auctionCfg.total_supply)}
                  <span class="text-muted font-medium"
                    >{tokenDisplay.token.symbol}</span
                  >
                </div>
              </div>
              <div class="flex items-center justify-between">
                <div class="text-muted text-xs font-medium">Min/Max Bid</div>
                <div class="text-sm font-bold">
                  {currencyDisplay.displayValue(auctionCfg.min_amount)}
                  <span class="text-muted mx-1 font-normal">~</span>
                  {currencyDisplay.displayValue(auctionCfg.max_amount)}
                </div>
              </div>
              <div class="flex items-center justify-between">
                <div class="text-muted text-xs font-medium">Finalize</div>
                <div class="text-right text-sm font-bold">
                  {#if finalizeKind[1]}
                    <a
                      class="text-indigo-500 transition-colors hover:text-indigo-400"
                      href={finalizeKind[1]}
                      target="_blank"
                      rel="noreferrer"
                    >
                      {finalizeKind[0]}
                    </a>
                  {:else}
                    <span>{finalizeKind[0]}</span>
                  {/if}
                </div>
              </div>
            </div>

            <div class="mt-8 space-y-3">
              <div class="flex items-center justify-between text-xs">
                <div class="text-muted font-bold tracking-wider uppercase">
                  {formatDatetime(auctionCfg.start_time)} →<br />
                  {formatDatetime(auctionCfg.end_time)}
                </div>
                <div class="font-black text-indigo-500"
                  >{Math.floor(progress(auctionCfg) * 100)}%</div
                >
              </div>
              <div
                class="bg-surface h-2 overflow-hidden rounded-full shadow-inner"
              >
                <div
                  class="h-full rounded-full bg-linear-to-r from-indigo-500 via-purple-500 to-amber-500 transition-all duration-1000"
                  style={`width:${Math.floor(progress(auctionCfg) * 100)}%`}
                ></div>
              </div>
            </div>
          {:else}
            <div class="flex flex-1 items-center justify-center">
              <p class="text-muted text-sm italic">Auction unconfigured</p>
            </div>
          {/if}
        </div>
      </section>

      <!-- Live Stats & Bidding Section -->
      <section class="grid gap-6 lg:grid-cols-2">
        <!-- Live Stats Card -->
        <div class="glass-border bg-surface/30 rounded-3xl p-6 md:p-8">
          <div class="flex items-center justify-between">
            <div class="space-y-1">
              <div
                class="text-xs font-bold tracking-widest text-indigo-500 uppercase"
                >Live Stats</div
              >
              <h2 class="text-xl font-bold tracking-tight">Market Clearing</h2>
            </div>
          </div>

          <div class="mt-6 grid gap-4 sm:grid-cols-2">
            {#each [{ label: 'Cumulative Raised', value: auctionInfo ? `${currencyDisplay.displayValue(auctionInfo.cumulative_demand_raised)} ${currencyInfo.symbol}` : '—' }, { label: 'Supply Released', value: auctionInfo ? `${tokenDisplay.displayValue(auctionInfo.cumulative_supply_released)} ${tokenInfo.symbol}` : '—' }, { label: 'Clearing Price', value: auctionInfo ? priceUnitsPerToken(auctionInfo.clearing_price) : priceUnitsPerToken(floorPrice) }, { label: 'Total Committed', value: auctionInfo ? `${currencyDisplay.displayValue(auctionInfo.total_amount)} ${currencyInfo.symbol}` : '—' }] as stat}
              <div
                class="glass-border bg-surface/40 hover:bg-surface/60 rounded-2xl p-4 transition-colors"
              >
                <div
                  class="text-muted text-xs font-bold tracking-widest uppercase"
                  >{stat.label}</div
                >
                <div class="mt-1 truncate text-base font-bold tracking-tight">
                  {stat.value}
                </div>
              </div>
            {/each}
          </div>

          <div class="mt-8">
            <div class="flex items-center justify-between">
              <div
                class="text-muted text-xs font-bold tracking-widest uppercase"
                >Demand Distribution</div
              >
              <div class="flex items-center gap-2">
                <span class="text-muted text-xs font-bold uppercase"
                  >Bucket</span
                >
                <Dropdown
                  open={openPriceBucket}
                  trigger={priceBucketTrigger}
                  triggerClass="px-2 py-1 bg-surface/50 rounded-lg border border-border-subtle hover:border-indigo-500/50 transition-colors"
                  menuClass="top-full mt-2 w-32 rounded-xl border border-white/10 bg-card/95 shadow-2xl backdrop-blur-xl"
                >
                  <ul class="py-2">
                    {#each [1n, 5n, 10n, 50n, 100n] as multiplier}
                      {@const precision = floorGroupedPrecision * multiplier}
                      <li>
                        <button
                          disabled={groupedPrecision == precision}
                          onclick={() => handlePriceBucket(precision)}
                          class="w-full px-4 py-2 text-left text-sm transition-colors hover:bg-indigo-500/10 hover:text-indigo-500 disabled:cursor-not-allowed disabled:opacity-30"
                        >
                          <span>{currencyDisplay.displayValue(precision)}</span>
                        </button>
                      </li>
                    {/each}
                  </ul>
                </Dropdown>
              </div>
            </div>

            {#if grouped.length === 0}
              <div
                class="bg-surface/20 mt-4 rounded-2xl border border-dashed border-white/5 py-10 text-center"
              >
                <p class="text-muted text-sm italic">No visualized data yet</p>
              </div>
            {:else}
              {@const maxBucket = grouped.reduce(
                (m, x) => (x[1] > m ? x[1] : m),
                0n
              )}
              <div class="mt-6 space-y-4">
                {#each grouped as [p, a]}
                  <div class="group space-y-1.5">
                    <div class="flex items-center justify-between text-xs">
                      <span class="text-muted font-bold uppercase"
                        >≥ {currencyDisplay.displayValue(p)}</span
                      >
                      <span class="font-bold"
                        >{currencyDisplay.displayValue(a)}
                        {currencyInfo.symbol}</span
                      >
                    </div>
                    <div
                      class="bg-surface h-2 overflow-hidden rounded-full shadow-inner"
                    >
                      <div
                        class="h-full rounded-full bg-linear-to-r from-indigo-500 to-purple-500 transition-all duration-1000 group-hover:from-indigo-400 group-hover:to-purple-400"
                        style={`width:${maxBucket === 0n ? 0 : Number((a * 100n) / maxBucket)}%`}
                      ></div>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <!-- Bidding Card -->
        <div class="glass-border bg-surface/30 rounded-3xl p-6 md:p-8">
          <div class="space-y-1">
            <div
              class="text-xs font-bold tracking-widest text-indigo-500 uppercase"
              >Participation</div
            >
            <h2 class="text-xl font-bold tracking-tight">Place a Bid</h2>
          </div>

          {#if !auctionInfo || !auctionCfg}
            <div class="flex h-64 items-center justify-center">
              <p class="text-muted text-sm italic">Auction not ready</p>
            </div>
          {:else}
            <div class="mt-6 space-y-6">
              <div
                class="flex items-center justify-between rounded-2xl border border-indigo-500/20 bg-indigo-500/5 p-4"
              >
                <div class="space-y-0.5">
                  <div
                    class="text-muted text-xs font-bold tracking-widest uppercase"
                    >Available Balance</div
                  >
                  <div class="text-lg font-bold tracking-tight">
                    {currencyDisplay.displayValue(myInfo.currency_amount)}
                    <span class="text-muted text-sm font-medium"
                      >{currencyInfo.symbol}</span
                    >
                  </div>
                </div>
                <button
                  class="rounded-xl bg-indigo-500 px-4 py-2 text-xs font-bold text-white uppercase shadow-lg shadow-indigo-500/20 transition-all hover:bg-indigo-600 disabled:opacity-50"
                  onclick={onUserFundsModal}
                  disabled={!stateInfo || !authStore.identity.isAuthenticated}
                >
                  {phase === 'ending' || phase === 'ended'
                    ? 'Manage'
                    : 'Deposit'}
                </button>
              </div>

              <div class="space-y-5">
                <div class="space-y-2">
                  <label
                    class="text-muted text-xs font-bold tracking-widest uppercase"
                    for="bidAmount"
                  >
                    Bid Amount
                  </label>
                  <div class="relative">
                    <input
                      id="bidAmount"
                      class="bg-surface/50 border-border-subtle w-full rounded-2xl border px-5 py-4 text-base font-bold transition-all focus:border-indigo-500/50 focus:ring-4 focus:ring-indigo-500/5 focus:outline-none"
                      placeholder={`Min: ${currencyDisplay.displayValue(auctionCfg.min_amount)}`}
                      bind:value={bidAmount}
                      onfocus={onBidAmountFocus}
                      disabled={phase != 'pre-bidding' && phase != 'bidding'}
                      inputmode="decimal"
                    />
                    <div
                      class="absolute top-1/2 right-4 flex -translate-y-1/2 items-center gap-3"
                    >
                      <span class="text-muted text-sm font-bold"
                        >{currencyInfo.symbol}</span
                      >
                      <button
                        class="text-xs font-black text-indigo-500 uppercase transition-colors hover:text-indigo-400 disabled:opacity-30"
                        onclick={() =>
                          (bidAmount = currencyDisplay.displayValue(
                            myInfo.currency_amount
                          ))}
                        disabled={myInfo.currency_amount === 0n ||
                          (phase !== 'pre-bidding' && phase !== 'bidding')}
                      >
                        Max
                      </button>
                    </div>
                  </div>
                </div>

                <div class="space-y-2">
                  <label
                    class="text-muted text-xs font-bold tracking-widest uppercase"
                    for="bidMaxPrice"
                  >
                    Max Price Limit
                  </label>
                  <div class="relative">
                    <input
                      id="bidMaxPrice"
                      class="bg-surface/50 border-border-subtle w-full rounded-2xl border px-5 py-4 text-base font-bold transition-all focus:border-indigo-500/50 focus:ring-4 focus:ring-indigo-500/5 focus:outline-none"
                      placeholder="0.00"
                      bind:value={bidMaxPrice}
                      onfocus={estimateMaxPrice}
                      disabled={phase != 'pre-bidding' && phase != 'bidding'}
                      inputmode="decimal"
                    />
                    <div
                      class="absolute top-1/2 right-4 -translate-y-1/2 text-sm font-bold"
                    >
                      <span class="text-muted"
                        >{currencyInfo.symbol}/{tokenInfo.symbol}</span
                      >
                    </div>
                  </div>
                  <p class="text-muted px-1 text-xs leading-relaxed">
                    The maximum price you are willing to pay per token. If the
                    clearing price exceeds this, you will be outbid.
                  </p>
                </div>

                <div class="pt-2">
                  {#if phase === 'ending' || phase === 'ended'}
                    <div
                      class="bg-surface/50 border-border-subtle text-muted flex w-full items-center justify-center gap-3 rounded-2xl border py-4 text-sm font-bold tracking-widest uppercase"
                    >
                      <span class="relative flex h-2.5 w-2.5">
                        <span
                          class="absolute inline-flex h-full w-full animate-ping rounded-full bg-red-400 opacity-75"
                        ></span>
                        <span
                          class="relative inline-flex h-2.5 w-2.5 rounded-full bg-red-500"
                        ></span>
                      </span>
                      {phase === 'ending' ? 'Bidding Closed' : 'Auction Ended'}
                    </div>
                  {:else}
                    <div class="flex gap-3">
                      <Button
                        class="glass-border bg-surface/50 hover:bg-surface text-muted hover:text-foreground rounded-2xl px-6 py-4 text-xs font-bold tracking-widest uppercase transition-all"
                        onclick={estimateMaxPrice}
                        isLoading={isEstimating}
                        disabled={bidAmountUnits == 0n}
                      >
                        Estimate
                      </Button>

                      {#if authStore.identity.isAuthenticated}
                        {#if myInfo.currency_amount < auctionCfg.min_amount || bidAmountUnits > myInfo.currency_amount}
                          <Button
                            class="flex-1 rounded-2xl bg-indigo-500 px-6 py-4 text-xs font-bold tracking-widest text-white uppercase shadow-lg shadow-indigo-500/25 transition-all hover:bg-indigo-600"
                            onclick={onUserFundsModal}
                          >
                            Deposit to Bid
                          </Button>
                        {:else}
                          <Button
                            class="bg-foreground text-background flex-1 rounded-2xl px-6 py-4 text-xs font-bold tracking-widest uppercase shadow-xl transition-all hover:opacity-90"
                            onclick={submitBid}
                            isLoading={isBidding}
                            disabled={!isBidable}
                          >
                            Submit Bid
                          </Button>
                        {/if}
                      {:else}
                        <Button
                          class="bg-foreground text-background flex-1 rounded-2xl px-6 py-4 text-xs font-bold tracking-widest uppercase shadow-xl transition-all hover:opacity-90"
                          onclick={onSignWith}
                          isLoading={isSigningIn}
                        >
                          Sign in to Participate
                        </Button>
                      {/if}
                    </div>
                  {/if}
                </div>

                {#if bidAmountErr || bidMaxPriceErr || estimateClearingPrice}
                  <div
                    class="animate-in fade-in slide-in-from-top-2 rounded-2xl p-4 transition-all {bidAmountErr ||
                    bidMaxPriceErr
                      ? 'border border-red-500/20 bg-red-500/5'
                      : 'border border-indigo-500/20 bg-indigo-500/5'}"
                  >
                    {#if bidAmountErr || bidMaxPriceErr}
                      <p class="text-xs font-bold text-red-500">
                        {bidAmountErr || bidMaxPriceErr}
                      </p>
                    {:else if estimateClearingPrice}
                      <div class="flex items-center justify-between">
                        <span
                          class="text-muted text-xs font-bold tracking-widest uppercase"
                          >Est. Clearing Price</span
                        >
                        <span class="text-sm font-black text-indigo-500">
                          {estimateClearingPrice}
                          {currencyInfo.symbol}/{tokenInfo.symbol}
                        </span>
                      </div>
                    {/if}
                  </div>
                {/if}

                <div class="bg-surface/30 rounded-2xl p-4">
                  <p class="text-muted text-xs leading-relaxed">
                    <strong class="text-foreground font-bold"
                      >Fair Launch Protocol:</strong
                    >
                    Bids are streamed linearly. Early entry maximizes your allocation.
                    Bids are final and cannot be cancelled, but unspent funds are
                    always refundable if you are outbid.
                  </p>
                </div>
              </div>
            </div>
          {/if}
        </div>
      </section>

      <!-- Price Discovery Chart -->
      <div class="glass-border bg-surface/20 rounded-3xl p-2 md:p-4">
        <PriceDiscoveryChart
          {snapshots}
          {auctionInfo}
          {auctionCfg}
          {currencyInfo}
          {currencyDisplay}
          {priceUnitsPerToken}
        />
      </div>

      <!-- My Activity Section -->
      <section class="glass-border bg-surface/30 rounded-3xl p-6 md:p-8">
        <div
          class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between"
        >
          <div>
            <div
              class="text-xs font-bold tracking-widest text-indigo-500 uppercase"
              >Account</div
            >
            <h2 class="text-2xl font-bold tracking-tight">My Activity</h2>
          </div>
          <div class="flex items-center gap-3">
            <button
              class="glass-border bg-surface/50 hover:bg-surface text-muted hover:text-foreground rounded-xl px-5 py-2.5 text-xs font-bold tracking-widest uppercase transition-all"
              onclick={onUserFundsModal}
              disabled={!stateInfo || !authStore.identity.isAuthenticated}
            >
              My Funds
            </button>
            <Button
              class="bg-foreground text-background rounded-xl px-5 py-2.5 text-xs font-bold tracking-widest uppercase shadow-lg transition-all hover:opacity-90"
              onclick={() => toastRun(claimAll, 'claim all failed')}
              disabled={!authStore.identity.isAuthenticated ||
                myBids.length === 0}
            >
              Claim All
            </Button>
          </div>
        </div>

        <div class="mt-8">
          {#if !auctionInfo || myBids.length === 0}
            <div
              class="bg-surface/20 rounded-2xl border border-dashed border-white/5 py-16 text-center"
            >
              <p class="text-muted text-sm italic"
                >No participation history found.</p
              >
            </div>
          {:else}
            <div class="glass-border overflow-hidden rounded-2xl">
              <div class="overflow-x-auto">
                <table class="w-full border-collapse text-left">
                  <thead>
                    <tr
                      class="bg-surface/50 text-muted text-xs font-bold tracking-widest uppercase"
                    >
                      <th class="px-6 py-4">ID</th>
                      <th class="px-6 py-4">Amount</th>
                      <th class="px-6 py-4">Max Price</th>
                      <th class="px-6 py-4">Status</th>
                      <th class="px-6 py-4 text-right">Action</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-white/5">
                    {#each myBids as b (b.id)}
                      {@const isOutbid = b.outbid_time.length === 1}
                      {@const isClaimed = b.claim_time > 0n}
                      {@const isClaimable =
                        !isClaimed &&
                        ((isOutbid && auctionInfo.is_graduated) ||
                          phase == 'ended')}
                      <tr class="hover:bg-surface/40 group transition-colors">
                        <td class="px-6 py-4">
                          <span
                            class="font-mono text-xs font-bold text-indigo-500"
                            >#{b.id.toString()}</span
                          >
                        </td>
                        <td class="px-6 py-4">
                          <div class="text-sm font-bold">
                            {currencyDisplay.displayValue(b.amount)}
                            <span class="text-muted text-xs font-medium"
                              >{currencyInfo.symbol}</span
                            >
                          </div>
                        </td>
                        <td class="px-6 py-4">
                          <div class="text-muted text-xs font-medium">
                            {priceUnitsPerToken(b.max_price)}
                          </div>
                        </td>
                        <td class="px-6 py-4">
                          {#if isClaimed}
                            <span
                              class="inline-flex items-center rounded-full bg-emerald-500/10 px-2.5 py-0.5 text-xs font-bold text-emerald-500 uppercase"
                            >
                              Claimed
                            </span>
                          {:else if isOutbid}
                            <span
                              class="inline-flex items-center rounded-full bg-red-500/10 px-2.5 py-0.5 text-xs font-bold text-red-500 uppercase"
                            >
                              Outbid
                            </span>
                          {:else if phase === 'ended'}
                            <span
                              class="inline-flex items-center rounded-full bg-indigo-500/10 px-2.5 py-0.5 text-xs font-bold text-indigo-500 uppercase"
                            >
                              Settled
                            </span>
                          {:else}
                            <span
                              class="inline-flex items-center rounded-full bg-amber-500/10 px-2.5 py-0.5 text-xs font-bold text-amber-500 uppercase"
                            >
                              Active
                            </span>
                          {/if}
                        </td>
                        <td class="px-6 py-4 text-right">
                          {#if !isClaimed}
                            <Button
                              class="glass-border bg-surface/50 hover:bg-surface text-muted hover:text-foreground rounded-lg px-3 py-1.5 text-xs font-bold uppercase transition-all disabled:opacity-30"
                              onclick={() =>
                                toastRun(() => claimOne(b.id), 'claim failed')}
                              disabled={!isClaimable}
                            >
                              Claim
                            </Button>
                          {/if}
                        </td>
                      </tr>
                      {#if b.tokens_filled > 0n || b.refund > 0n}
                        <tr class="bg-indigo-500/2">
                          <td colspan="5" class="px-6 py-3">
                            <div class="flex items-center gap-6 text-xs">
                              <div class="flex items-center gap-2">
                                <span
                                  class="text-muted font-bold tracking-widest uppercase"
                                  >Filled</span
                                >
                                <span class="text-foreground font-black">
                                  {tokenDisplay.displayValue(b.tokens_filled)}
                                  {tokenInfo.symbol}
                                </span>
                              </div>
                              <div class="h-3 w-px bg-white/10"></div>
                              <div class="flex items-center gap-2">
                                <span
                                  class="text-muted font-bold tracking-widest uppercase"
                                  >Refund</span
                                >
                                <span class="text-foreground font-black">
                                  {currencyDisplay.displayValue(b.refund)}
                                  {currencyInfo.symbol}
                                </span>
                              </div>
                            </div>
                          </td>
                        </tr>
                      {/if}
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>
          {/if}
        </div>
      </section>
    {/if}
  </main>
</div>
