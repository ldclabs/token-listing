<script lang="ts">
  import type {
    AuctionId,
    AuctionInfo
  } from '$declarations/token_listing_canister/token_listing_canister.did'
  import { tokenListingActor } from '$lib/canisters/tokenListing'
  import Header from '$lib/components/Header.svelte'
  import { TOKEN_LISTING } from '$lib/constants'
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
    class="relative z-10 mx-auto w-full max-w-6xl space-y-6 px-4 py-6 md:px-8 md:py-10"
  >
    <section
      class="glass-border relative overflow-hidden rounded-xl p-4 md:p-6"
    >
      <div class="pointer-events-none absolute inset-0" aria-hidden="true">
        <div
          class="absolute -top-24 -left-24 h-56 w-56 rounded-full bg-purple-500/10 blur-3xl"
        ></div>
        <div
          class="absolute -right-24 -bottom-24 h-64 w-64 rounded-full bg-amber-500/10 blur-3xl"
        ></div>
      </div>

      <div class="relative">
        <div
          class="flex flex-col gap-3 md:flex-row md:items-start md:justify-between"
        >
          <div class="min-w-0">
            <div
              class="text-muted text-xs font-semibold tracking-wide uppercase"
            >
              Launchpad
            </div>
            <div class="mt-1 text-lg font-bold md:text-2xl">
              Token Launches, Finally Fair.
            </div>
            <div class="text-muted mt-2 max-w-2xl text-sm leading-relaxed">
              Say goodbye to bot snipers, gas wars, and rushed decision-making.
              Tokenlist.ing utilizes the Continuous Clearing Auction (CCA)
              mechanism to democratize price discovery.
            </div>
            <div class="text-muted mt-2 max-w-2xl text-sm leading-relaxed">
              Unlike traditional Fixed-Price or Dutch auctions where timing is
              everything, CCA allows the market to find the true price naturally
              over time.
            </div>

            <div class="mt-3 flex flex-wrap gap-2">
              <span
                class="border-border-subtle bg-surface text-muted inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] font-semibold tracking-wide uppercase"
              >
                Continuous Clearing Auction
              </span>
              <span
                class="border-border-subtle bg-surface text-muted inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] font-semibold tracking-wide uppercase"
              >
                Bot Resistant
              </span>
              <span
                class="border-border-subtle bg-surface text-muted inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] font-semibold tracking-wide uppercase"
              >
                Fully On Chain
              </span>
            </div>
          </div>
        </div>

        <div class="mt-5 grid gap-3 lg:grid-cols-3">
          <div class="border-border-subtle bg-surface rounded-lg border p-4">
            <div class="text-xs font-semibold tracking-wide uppercase">
              No Timing Games
            </div>
            <div class="text-muted mt-1 text-sm leading-relaxed">
              Your entry price depends on valuation, not how fast you click.
            </div>
          </div>

          <div class="border-border-subtle bg-surface rounded-lg border p-4">
            <div class="text-xs font-semibold tracking-wide uppercase">
              Bot Resistant
            </div>
            <div class="text-muted mt-1 text-sm leading-relaxed">
              Sniping is mathematically impossible.
            </div>
          </div>

          <div class="border-border-subtle bg-surface rounded-lg border p-4">
            <div class="text-xs font-semibold tracking-wide uppercase">
              Millisecond Precision
            </div>
            <div class="text-muted mt-1 text-sm leading-relaxed">
              Powered by ICP, enjoying a seamless, real-time auction experience.
            </div>
          </div>
        </div>

        <div class="mt-3 grid gap-3 md:grid-cols-2">
          <div class="p-4">
            <div class="flex items-center justify-between gap-3">
              <div class="text-xs font-semibold tracking-wide uppercase">
                How CCA Works
              </div>
              <div
                class="text-muted text-[10px] font-semibold tracking-wide uppercase"
              >
                Time-Weighted Allocation
              </div>
            </div>

            <div class="mt-3 grid gap-2">
              <div class="flex items-start gap-3">
                <div
                  class="border-border-subtle bg-surface text-muted flex h-7 w-7 shrink-0 items-center justify-center rounded-full border text-[10px] font-semibold"
                >
                  1
                </div>
                <div class="min-w-0">
                  <div class="text-sm font-semibold">Place bids over time</div>
                  <div class="text-muted mt-0.5 text-xs leading-relaxed">
                    Decide based on valuation, not speed.
                  </div>
                </div>
              </div>
              <div class="flex items-start gap-3">
                <div
                  class="border-border-subtle bg-surface text-muted flex h-7 w-7 shrink-0 items-center justify-center rounded-full border text-[10px] font-semibold"
                >
                  2
                </div>
                <div class="min-w-0">
                  <div class="text-sm font-semibold">Demand accumulates</div>
                  <div class="text-muted mt-0.5 text-xs leading-relaxed">
                    The book aggregates naturally as participants join.
                  </div>
                </div>
              </div>
              <div class="flex items-start gap-3">
                <div
                  class="border-border-subtle bg-surface text-muted flex h-7 w-7 shrink-0 items-center justify-center rounded-full border text-[10px] font-semibold"
                >
                  3
                </div>
                <div class="min-w-0">
                  <div class="text-sm font-semibold">Market clears fairly</div>
                  <div class="text-muted mt-0.5 text-xs leading-relaxed">
                    Allocation is determined by accumulated participation.
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="p-4">
            <div class="text-xs font-semibold tracking-wide uppercase">
              Integral CCA (Accumulator)
            </div>
            <div class="text-muted mt-1 text-xs leading-relaxed">
              Your effective entry is time-weighted by the price path:
              allocation accumulates over time, not from the last-tick price.
            </div>

            <div class="mt-4">
              <svg
                class="text-muted w-full"
                viewBox="0 0 360 150"
                fill="none"
                aria-label="Integral CCA illustration: price path, time-weighted average, and accumulated token allocation"
                role="img"
              >
                <!-- Single panel -->
                <rect
                  x="18"
                  y="14"
                  width="324"
                  height="112"
                  rx="10"
                  class="fill-current"
                  opacity="0.06"
                />

                <!-- Axes (shared time axis) -->
                <path
                  d="M36 24 V126 H332"
                  class="stroke-current"
                  stroke-width="1"
                  opacity="0.35"
                />
                <path
                  d="M36 46 H332 M36 68 H332 M36 90 H332 M36 112 H332"
                  class="stroke-current"
                  stroke-width="1"
                  opacity="0.12"
                />

                <!-- Price path area (integral) -->
                <g class="text-purple-500/60">
                  <path
                    d="M48 115 L90 115 L90 95 L140 95 L140 75 L190 75 L190 85 L240 85 L240 65 L304 65 L304 126 L48 126 Z"
                    class="fill-current"
                    opacity="0.18"
                  />
                  <path
                    d="M48 115 L90 115 L90 95 L140 95 L140 75 L190 75 L190 85 L240 85 L240 65 L304 65"
                    class="stroke-current"
                    stroke-width="2.5"
                    stroke-linejoin="round"
                    stroke-linecap="round"
                  />
                </g>

                <!-- Average price (time-weighted) guide line -->
                <g class="text-amber-500/60">
                  <path
                    d="M48 88 H304"
                    class="stroke-current"
                    stroke-width="2"
                    stroke-dasharray="6 4"
                    stroke-linecap="round"
                    opacity="0.9"
                  />
                </g>

                <!-- Accumulator / cumulative allocation curve -->
                <g class="text-yellow-500/40">
                  <path
                    d="M48 105 L90 90 L140 70 L190 55 L240 45 L304 25"
                    class="stroke-current"
                    stroke-width="2.5"
                    stroke-linejoin="round"
                    stroke-linecap="round"
                  />
                  <path
                    d="M48 105 L90 90 L140 70 L190 55 L240 45 L304 25 L304 126 L48 126 Z"
                    class="fill-current"
                    opacity="0.10"
                  />
                </g>

                <!-- Labels -->
                <text
                  x="36"
                  y="16"
                  class="fill-current"
                  font-size="10"
                  opacity="0.85"
                >
                  Price path P(t) + Accumulator A(t)
                </text>
                <text
                  x="252"
                  y="84"
                  class="fill-current"
                  font-size="10"
                  opacity="0.85"
                >
                  Avg entry
                </text>

                <text
                  x="284"
                  y="144"
                  class="fill-current"
                  font-size="10"
                  opacity="0.7"
                >
                  time →
                </text>
              </svg>

              <div
                class="mt-3 flex flex-wrap items-center gap-x-4 gap-y-2 text-[10px] font-semibold tracking-wide uppercase"
              >
                <div class="text-muted inline-flex items-center gap-2">
                  <span class="h-2 w-2 rounded-full bg-purple-500/40"></span>
                  <span>Price Path</span>
                </div>
                <div class="text-muted inline-flex items-center gap-2">
                  <span class="h-2 w-2 rounded-full bg-amber-500/40"></span>
                  <span>Time-Weighted Avg</span>
                </div>
                <div class="text-muted inline-flex items-center gap-2">
                  <span class="h-2 w-2 rounded-full bg-yellow-500/30"></span>
                  <span>Accumulated Tokens</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    {#if isLoading}
      <div
        class="glass-border flex items-center justify-center rounded-xl p-10"
      >
        <Spinner class="h-6 w-6" />
      </div>
    {:else if auctions.length === 0}
      <div class="glass-border rounded-xl p-6">
        <div class="text-muted text-sm">No auctions yet.</div>
      </div>
    {:else}
      <section class="grid gap-4 md:grid-cols-2">
        {#each auctions as a (auctionIdText(a.id))}
          {@const id = auctionIdText(a.id)}
          <a
            class="glass-border group relative overflow-hidden rounded-xl p-4 transition-all duration-300 hover:-translate-y-1 hover:shadow-lg md:p-6"
            href={`/_/launchpad/${id}`}
          >
            <div
              class="flex flex-col items-start justify-between gap-3 lg:flex-row"
            >
              <div class="flex min-w-0 items-center gap-3">
                <div
                  class="bg-surface relative h-10 w-10 shrink-0 overflow-hidden rounded-full"
                >
                  {#if a.token_logo_url}
                    <img
                      class="h-full w-full object-cover"
                      src={a.token_logo_url}
                      alt={a.token_symbol}
                      loading="lazy"
                    />
                  {/if}
                </div>
                <div class="min-w-0">
                  <div class="flex items-center gap-2">
                    <div class="truncate font-bold">
                      {a.name || 'Auction'}
                      <span class="text-muted font-semibold"
                        >· {a.token_symbol}</span
                      >
                    </div>
                  </div>
                  <div class="text-muted mt-0.5 truncate text-xs">
                    <span>Required: {displayRequired(a)}</span>
                  </div>
                </div>
              </div>

              <div class="flex items-center gap-1">
                {#if a.is_graduated}
                  <span
                    class="border-border-subtle bg-surface text-muted inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] font-semibold tracking-wide uppercase"
                  >
                    Graduated
                  </span>
                {/if}
                <span
                  class="border-border-subtle bg-surface text-muted inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] font-semibold tracking-wide uppercase"
                >
                  {chainLabel(a.chain)}
                </span>
                <span
                  class="border-border-subtle bg-surface text-muted inline-flex shrink-0 items-center rounded-full border px-2 py-0.5 text-[10px] font-semibold tracking-wide uppercase"
                >
                  {statusLabel(a)}
                </span>
              </div>
            </div>

            <div class="text-muted mt-3 line-clamp-3 text-sm leading-relaxed">
              {a.description || '—'}
            </div>

            <div class="mt-4 grid gap-3 sm:grid-cols-2">
              <div
                class="border-border-subtle bg-surface rounded-lg border p-3"
              >
                <div class="text-muted text-xs">Cumulative Raised</div>
                <div class="mt-1 text-sm font-bold">{displayRaised(a)}</div>
              </div>
              <div
                class="border-border-subtle bg-surface rounded-lg border p-3"
              >
                <div class="text-muted text-xs">Supply Released</div>
                <div class="mt-1 text-sm font-bold">{displayReleased(a)}</div>
              </div>
              <div
                class="border-border-subtle bg-surface rounded-lg border p-3"
              >
                <div class="text-muted text-xs">Clearing Price</div>
                <div class="mt-1 text-sm font-bold"
                  >{displayClearingPrice(a)}</div
                >
              </div>
              <div
                class="border-border-subtle bg-surface rounded-lg border p-3"
              >
                <div class="text-muted text-xs">Bids</div>
                <div class="mt-1 text-sm font-bold"
                  >{a.bids_count.toString()}</div
                >
              </div>
            </div>

            <div
              class="text-muted mt-4 flex items-center justify-between gap-3 text-xs"
            >
              <div class="truncate">
                {formatDatetime(a.start_time)} → {formatDatetime(a.end_time)}
              </div>
              <div
                class="shrink-0 transition-transform group-hover:translate-x-1"
              >
                →
              </div>
            </div>
          </a>
        {/each}
      </section>
    {/if}
  </main>
</div>
