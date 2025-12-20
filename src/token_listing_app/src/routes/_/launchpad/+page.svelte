<script lang="ts">
  import type {
    AuctionId,
    AuctionInfo
  } from '$declarations/token_listing_canister/token_listing_canister.did'
  import { tokenListingActor } from '$lib/canisters/tokenListing'
  import Header from '$lib/components/Header.svelte'
  import { TOKEN_LISTING } from '$lib/constants'
  import ArrowRightUpLine from '$lib/icons/arrow-right-up-line.svelte'
  import { toastRun } from '$lib/stores/toast.svelte'
  import Spinner from '$lib/ui/Spinner.svelte'
  import { chainLabel } from '$lib/utils/chain'
  import { formatDatetime } from '$lib/utils/helper'
  import { TokenDisplay, type TokenInfo } from '$lib/utils/token'
  import { onMount } from 'svelte'

  const listingActor = tokenListingActor(TOKEN_LISTING)
  let auctions = $state<AuctionInfo[]>([])
  let isLoading = $state(true)

  function auctionIdText(id: AuctionId): string {
    if ('Icp' in id) return id.Icp
    if ('Sol' in id) return id.Sol
    return id.Evm
  }

  function statusLabel(a: AuctionInfo): 'Upcoming' | 'Live' | 'Ended' {
    const n = BigInt(Date.now())
    if (n < a.start_time) return 'Upcoming'
    if (n < a.end_time) return 'Live'
    return 'Ended'
  }

  function tokenInfoFromAuction(a: AuctionInfo): TokenInfo {
    return {
      name: a.token_name,
      symbol: a.token_symbol,
      decimals: a.token_decimals,
      fee: 0n,
      one: 10n ** BigInt(a.token_decimals),
      logo: a.token_logo_url,
      address: a.token
    }
  }

  function currencyInfoFromAuction(a: AuctionInfo): TokenInfo {
    return {
      name: a.currency_name,
      symbol: a.currency_symbol,
      decimals: a.currency_decimals,
      fee: 0n,
      one: 10n ** BigInt(a.currency_decimals),
      logo: a.currency_logo_url,
      address: a.currency
    }
  }

  function displayRequired(a: AuctionInfo): string {
    const currency = currencyInfoFromAuction(a)
    const d = new TokenDisplay(currency, 0n)
    return `${d.displayValue(a.required_currency_raised)} ${currency.symbol}`
  }

  function displayRaised(a: AuctionInfo): string {
    const currency = currencyInfoFromAuction(a)
    const d = new TokenDisplay(currency, 0n)
    return `${d.displayValue(a.total_demand_raised)} ${currency.symbol}`
  }

  function displayReleased(a: AuctionInfo): string {
    const token = tokenInfoFromAuction(a)
    const d = new TokenDisplay(token, 0n)
    return `${d.displayValue(a.total_supply_released)} ${token.symbol}`
  }

  function displayClearingPrice(a: AuctionInfo): string {
    const token = tokenInfoFromAuction(a)
    const currency = currencyInfoFromAuction(a)
    const d = new TokenDisplay(currency, 0n)
    return `${d.displayValue(a.clearing_price)} ${currency.symbol}/${token.symbol}`
  }

  onMount(() => {
    return toastRun(async (_signal, _abortingQue) => {
      isLoading = true
      auctions = await listingActor.list_auctions(20n, [])
      isLoading = false
    }).abort
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

  <Header description={'Launchpad'} />

  <main
    class="relative z-10 mx-auto w-full max-w-6xl space-y-12 px-4 py-10 md:px-8 md:py-16"
  >
    <!-- Concise Hero Section -->
    <section class="relative">
      <div class="max-w-3xl space-y-4">
        <div
          class="text-xs font-bold tracking-[0.2em] text-indigo-500 uppercase"
        >
          Launchpad
        </div>
        <h1
          class="font-serif text-4xl font-bold tracking-tight sm:text-5xl lg:text-6xl"
        >
          Token Launches, <span class="gradient-text">Finally Fair</span>.
        </h1>
        <p class="text-muted text-lg leading-relaxed md:text-xl">
          Discover and participate in sovereign project launches powered by the
          <strong class="text-foreground font-semibold"
            >Continuous Clearing Auction</strong
          >.
        </p>
      </div>
    </section>

    <!-- Auction Cards Section (Primary Action) -->
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <h2
          class="text-xs font-bold tracking-[0.2em] text-indigo-500 uppercase"
        >
          Active & Upcoming Auctions
        </h2>
      </div>

      {#if isLoading}
        <div class="flex h-64 items-center justify-center">
          <Spinner class="h-8 w-8 text-indigo-500" />
        </div>
      {:else if auctions.length === 0}
        <div class="glass-border bg-surface/20 rounded-2xl p-12 text-center">
          <p class="text-muted text-lg">No auctions yet.</p>
        </div>
      {:else}
        <section class="grid gap-6 md:grid-cols-2">
          {#each auctions as a (auctionIdText(a.id))}
            {@const id = auctionIdText(a.id)}
            <a
              class="glass-border group bg-surface/30 hover:bg-surface/50 relative flex flex-col overflow-hidden rounded-2xl p-6 transition-all duration-500 hover:-translate-y-2 hover:shadow-2xl"
              href={`/_/launchpad/${id}`}
            >
              <div class="flex items-start justify-between gap-4">
                <div class="flex min-w-0 items-center gap-4">
                  <div
                    class="bg-surface ring-border-subtle relative h-12 w-12 shrink-0 overflow-hidden rounded-xl shadow-inner ring-1"
                  >
                    {#if a.token_logo_url}
                      <img
                        class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-110"
                        src={a.token_logo_url}
                        alt={a.token_symbol}
                        loading="lazy"
                      />
                    {/if}
                  </div>
                  <div class="min-w-0">
                    <h3 class="truncate text-lg font-bold tracking-tight">
                      {a.name || 'Auction'}
                    </h3>
                    <div
                      class="text-muted mt-1 flex items-center gap-2 text-xs font-medium"
                    >
                      <span class="text-indigo-500">{a.token_symbol}</span>
                      <span class="text-border-subtle">/</span>
                      <span>{a.currency_symbol}</span>
                    </div>
                  </div>
                </div>

                <div class="flex flex-wrap justify-end gap-1.5">
                  {#if a.is_graduated}
                    <span
                      class="rounded-full bg-emerald-500/10 px-2 py-0.5 text-xs font-bold tracking-wider text-emerald-500 uppercase"
                    >
                      Graduated
                    </span>
                  {/if}
                  <span
                    class="rounded-full bg-indigo-500/10 px-2 py-0.5 text-xs font-bold tracking-wider text-indigo-500 uppercase"
                  >
                    {chainLabel(a.chain)}
                  </span>
                  <span
                    class="bg-surface border-border-subtle text-muted rounded-full border px-2 py-0.5 text-xs font-bold tracking-wider uppercase"
                  >
                    {statusLabel(a)}
                  </span>
                </div>
              </div>

              <p class="text-muted mt-4 line-clamp-2 text-sm leading-relaxed">
                {a.description || 'No description provided.'}
              </p>

              <div class="mt-6 grid grid-cols-2 gap-3">
                {#each [{ label: 'Raised', value: displayRaised(a) }, { label: 'Released', value: displayReleased(a) }, { label: 'Price', value: displayClearingPrice(a) }, { label: 'Required', value: displayRequired(a) }] as stat}
                  <div class="bg-surface/50 rounded-xl p-3">
                    <p
                      class="text-muted text-xs font-bold tracking-widest uppercase"
                      >{stat.label}</p
                    >
                    <p class="mt-1 truncate text-sm font-bold tracking-tight"
                      >{stat.value}</p
                    >
                  </div>
                {/each}
              </div>

              <div
                class="border-border-subtle mt-6 flex items-center justify-between border-t pt-4"
              >
                <div
                  class="text-muted text-xs font-medium tracking-wider uppercase"
                >
                  {formatDatetime(a.start_time)} → {formatDatetime(a.end_time)}
                </div>
                <div
                  class="text-indigo-500 transition-transform duration-500 group-hover:translate-x-1"
                >
                  <ArrowRightUpLine />
                </div>
              </div>
            </a>
          {/each}
        </section>
      {/if}
    </div>

    <!-- Detailed CCA Explanation (Secondary Info) -->
    <section
      class="glass-border bg-surface/20 relative overflow-hidden rounded-3xl p-8 md:p-12"
    >
      <div class="pointer-events-none absolute inset-0" aria-hidden="true">
        <div
          class="absolute -top-40 -left-40 h-96 w-96 rounded-full bg-indigo-500/5 blur-[100px]"
        ></div>
        <div
          class="absolute -right-40 -bottom-40 h-96 w-96 rounded-full bg-amber-500/5 blur-[100px]"
        ></div>
      </div>

      <div class="relative">
        <div class="max-w-3xl space-y-6">
          <h2 class="font-serif text-3xl font-bold tracking-tight sm:text-4xl">
            How it works: <span class="gradient-text">The CCA Mechanism</span>
          </h2>
          <div class="text-muted space-y-4 text-base leading-relaxed">
            <p>
              Tokenlist.ing utilizes the <strong
                class="text-foreground font-semibold"
                >Continuous Clearing Auction (CCA)</strong
              >
              to ensure a fair launch for everyone. Unlike traditional auctions where
              timing is everything, CCA allows the market to find the true price naturally
              over time.
            </p>
          </div>
        </div>

        <div class="mt-10 grid gap-6 lg:grid-cols-3">
          {#each [{ title: 'No Timing Games', desc: 'Your entry price depends on valuation, not how fast you click.' }, { title: 'Bot Resistant', desc: 'Sniping is mathematically impossible due to time-weighted streaming.' }, { title: 'Millisecond Precision', desc: 'Powered by ICP, enjoying a seamless, real-time auction experience.' }] as feature}
            <div
              class="glass-border group bg-surface/40 hover:bg-surface/60 rounded-2xl p-6 transition-all duration-500"
            >
              <h3
                class="text-foreground text-sm font-bold tracking-widest uppercase"
              >
                {feature.title}
              </h3>
              <p class="text-muted mt-2 text-sm leading-relaxed">
                {feature.desc}
              </p>
            </div>
          {/each}
        </div>

        <div class="mt-12 grid gap-8 lg:grid-cols-2">
          <div class="space-y-8 p-2">
            <div class="space-y-2">
              <h3
                class="text-xs font-bold tracking-[0.2em] text-indigo-500 uppercase"
              >
                The Process
              </h3>
              <p
                class="text-muted text-sm font-medium tracking-wider uppercase"
              >
                Time-Weighted Allocation
              </p>
            </div>

            <div class="grid gap-6">
              {#each [{ step: 1, title: 'Place bids over time', desc: 'Decide based on valuation, not speed.' }, { step: 2, title: 'Demand accumulates', desc: 'The book aggregates naturally as participants join.' }, { step: 3, title: 'Market clears fairly', desc: 'Allocation is determined by accumulated participation.' }] as item}
                <div class="group flex items-start gap-5">
                  <div
                    class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-linear-to-br from-indigo-500 to-purple-600 text-sm font-bold text-white shadow-lg transition-transform duration-500 group-hover:scale-110"
                  >
                    {item.step}
                  </div>
                  <div class="space-y-1">
                    <h4 class="text-base font-bold tracking-tight"
                      >{item.title}</h4
                    >
                    <p class="text-muted text-sm leading-relaxed">
                      {item.desc}
                    </p>
                  </div>
                </div>
              {/each}
            </div>
          </div>

          <div class="glass-border bg-surface/30 rounded-2xl p-6 md:p-8">
            <div class="space-y-2">
              <h3
                class="text-foreground text-xs font-bold tracking-widest uppercase"
              >
                Integral CCA (Accumulator)
              </h3>
              <p class="text-muted text-xs leading-relaxed">
                Your effective entry is time-weighted by the price path:
                allocation accumulates over time, not from the last-tick price.
              </p>
            </div>

            <div class="mt-8">
              <svg
                class="text-muted w-full"
                viewBox="0 0 360 150"
                fill="none"
                aria-label="Integral CCA illustration"
                role="img"
              >
                <rect
                  x="0"
                  y="0"
                  width="360"
                  height="150"
                  rx="16"
                  class="fill-surface/50"
                />
                <path
                  d="M40 30 V120 H320"
                  class="stroke-current"
                  stroke-width="1"
                  opacity="0.2"
                />

                <!-- Price path -->
                <g class="text-indigo-500">
                  <path
                    d="M50 110 L90 110 L90 90 L140 90 L140 70 L190 70 L190 80 L240 80 L240 60 L310 60"
                    class="stroke-current"
                    stroke-width="2.5"
                    stroke-linejoin="round"
                    stroke-linecap="round"
                  />
                  <path
                    d="M50 110 L90 110 L90 90 L140 90 L140 70 L190 70 L190 80 L240 80 L240 60 L310 60 V120 H50 Z"
                    class="fill-current"
                    opacity="0.1"
                  />
                </g>

                <!-- Average price -->
                <line
                  x1="50"
                  y1="85"
                  x2="310"
                  y2="85"
                  class="stroke-amber-500"
                  stroke-width="2"
                  stroke-dasharray="6 4"
                  opacity="0.8"
                />

                <!-- Accumulator -->
                <g class="text-purple-500">
                  <path
                    d="M50 100 L90 85 L140 65 L190 50 L240 40 L310 20"
                    class="stroke-current"
                    stroke-width="2.5"
                    stroke-linejoin="round"
                    stroke-linecap="round"
                  />
                </g>
              </svg>

              <div class="mt-6 flex flex-wrap gap-4">
                {#each [{ color: 'bg-indigo-500', label: 'Price Path' }, { color: 'bg-amber-500', label: 'Time-Weighted Avg' }, { color: 'bg-purple-500', label: 'Accumulated Tokens' }] as legend}
                  <div class="flex items-center gap-2">
                    <span class="h-2 w-2 rounded-full {legend.color}"></span>
                    <span
                      class="text-muted text-xs font-bold tracking-wider uppercase"
                      >{legend.label}</span
                    >
                  </div>
                {/each}
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  </main>
</div>
