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
    <section class="glass-border rounded-xl p-4 md:p-6">
      <div class="flex items-end justify-between gap-4">
        <div>
          <div class="text-muted text-xs font-semibold tracking-wide uppercase">
            Auctions
          </div>
          <div class="text-lg font-bold">Launchpad</div>
          <div class="text-muted mt-1 text-sm">
            Browse active and upcoming Continuous Clearing Auctions.
          </div>
        </div>
        <div class="text-muted text-xs">Showing {auctions.length} items</div>
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
