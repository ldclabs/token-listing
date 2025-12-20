<script lang="ts">
  import type { AuctionInfo as Auction } from '$declarations/ic_auction/ic_auction.did'
  import type {
    AuctionId,
    AuctionInfo
  } from '$declarations/token_listing_canister/token_listing_canister.did'
  import { icAuctionActor } from '$lib/canisters/icAuction'
  import { tokenListingActor } from '$lib/canisters/tokenListing'
  import { TOKEN_LISTING } from '$lib/constants'
  import ArrowRightUpLine from '$lib/icons/arrow-right-up-line.svelte'
  import { getTheme } from '$lib/stores/theme.svelte'
  import { toastRun } from '$lib/stores/toast.svelte'
  import { chainLabel } from '$lib/utils/chain'
  import { formatDatetime } from '$lib/utils/helper'
  import { TokenDisplay, type TokenInfo } from '$lib/utils/token'
  import { isActive } from '$lib/utils/window'
  import Header from '$src/lib/components/Header.svelte'
  import { onDestroy, onMount } from 'svelte'

  const theme = getTheme()
  const listingActor = tokenListingActor(TOKEN_LISTING)

  let latestAuction = $state<AuctionInfo | null>(null)
  let auction = $state<Auction | null>(null)

  let nowMs = $state(Date.now())

  function auctionIdText(id: AuctionId): string {
    if ('Icp' in id) return id.Icp
    if ('Sol' in id) return id.Sol
    return id.Evm
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

  function statusLabel(a: AuctionInfo): 'Upcoming' | 'Active' | 'Ended' {
    const n = BigInt(nowMs)
    if (n < a.start_time) return 'Upcoming'
    if (n < a.end_time) return 'Active'
    return 'Ended'
  }

  function timeRemainingLabel(a: AuctionInfo): string {
    const endMs =
      nowMs > a.start_time ? Number(a.end_time) : Number(a.start_time)
    const delta = Math.max(0, endMs - nowMs)
    const totalMinutes = Math.floor(delta / 60000)
    const days = Math.floor(totalMinutes / (60 * 24))
    const hours = Math.floor((totalMinutes - days * 60 * 24) / 60)
    const minutes = Math.max(0, totalMinutes - days * 60 * 24 - hours * 60)
    if (days > 0) return `${days}d ${hours}h ${minutes}m`
    if (hours > 0) return `${hours}h ${minutes}m`
    return `${minutes}m`
  }

  const latest = $derived.by(() => {
    if (!latestAuction) return null
    const a = latestAuction
    const au = auction
    const token = tokenInfoFromAuction(a)
    const currency = currencyInfoFromAuction(a)
    const currencyDisplay = new TokenDisplay(currency, 0n)
    const requiredCurrency = currencyDisplay.displayValue(
      a.required_currency_raised
    )
    const clearingPriceValue = currencyDisplay.displayValue(
      au ? au.clearing_price : a.clearing_price
    )
    const raised = currencyDisplay.displayValue(
      au ? au.cumulative_demand_raised : a.total_demand_raised
    )
    const totalCommitted = au
      ? currencyDisplay.displayValue(au.total_amount)
      : '—'
    const bidsCount = (au ? au.bids_count : a.bids_count).toString()
    const totalBidders = au ? au.total_bidders.toString() : '—'
    const remaining = timeRemainingLabel(a)
    const status = statusLabel(a)
    const isActive = status === 'Active'
    const id = auctionIdText(a.id)
    const href = `/_/launchpad/${id}`
    return {
      a,
      token,
      currency,
      requiredCurrency,
      clearingPriceValue,
      raised,
      totalCommitted,
      bidsCount,
      totalBidders,
      remaining,
      status,
      isActive,
      href,
      total_demand_raised: a.total_demand_raised,
      chain: chainLabel(a.chain)
    }
  })

  const navLinks = [
    { label: 'Launchpad', href: '/_/launchpad' },
    { label: 'Features', href: '#features' },
    { label: 'Chains', href: '#chains' },
    { label: 'FAQ', href: '#faq' }
  ]

  const pillars = [
    {
      title: 'Permanent Token Identity',
      desc: "Your token's logo, name, and metadata stored forever on ICP blockchain—no IPFS pins to expire, no servers to fail.",
      tag: 'Metadata',
      icon: '🏛️'
    },
    {
      title: 'Cross-Chain Minting',
      desc: 'Issue tokens on EVM, Solana, or ICP from a single dashboard. Manage supply and permissions across all chains.',
      tag: 'Mint',
      icon: '🔗'
    },
    {
      title: 'Fair Continuous Clearing Auctions',
      desc: 'CCA spreads bids over time automatically. Early supporters get fair prices; bots and snipers get nothing.',
      tag: 'Launch',
      icon: '⚖️'
    },
    {
      title: 'On-Chain Governance',
      desc: 'Community-driven decisions with transparent, executable on-chain proposals. No hidden multisigs, no backroom deals.',
      tag: 'DAO',
      icon: '🗳️'
    }
  ]

  const steps = [
    {
      title: 'Set the Rules',
      body: 'Define auction duration, minimum raise target, and currency. The release curve adapts automatically.'
    },
    {
      title: 'Bids Flow In',
      body: 'Each bid is spread across remaining time—no advantage to waiting until the last second.'
    },
    {
      title: 'Price Finds Itself',
      body: 'The clearing price updates continuously based on total demand. Everyone pays the same fair price.'
    },
    {
      title: 'Success or Full Refund',
      body: 'Hit your funding target? Tokens distribute. Miss it? All funds return automatically. No stuck capital.'
    },
    {
      title: 'Liquidity Ready',
      body: 'Upon success, proceeds seed a trading pool instantly. Your token is live and tradeable immediately.'
    }
  ]

  const chains = [
    {
      name: 'ICP',
      note: 'Orchestration layer & permanent storage',
      color: 'from-purple-500 to-pink-500'
    },
    {
      name: 'EVM',
      note: 'Ethereum & EVM-compatible issuance and bridges',
      color: 'from-blue-500 to-cyan-500'
    },
    {
      name: 'Solana',
      note: 'High-speed SPL token support',
      color: 'from-green-400 to-emerald-500'
    },
    {
      name: 'More',
      note: 'Expanding to L1s and appchains',
      color: 'from-orange-400 to-yellow-500'
    }
  ]

  const faqs = [
    {
      q: 'Why store metadata on ICP?',
      a: "IPFS pins expire. Centralized servers go down. ICP provides permanent, HTTP-accessible storage—your token's identity lives as long as the blockchain does."
    },
    {
      q: 'What makes CCA different from other launchpads?',
      a: 'Traditional launches reward bots and insiders who can time transactions perfectly. CCA spreads every bid over time, so the market—not millisecond timing—determines who gets tokens and at what price.'
    },
    {
      q: 'How does pricing work?',
      a: "The clearing price is simply total demand divided by available supply at any moment. As more people bid, price rises. If bids exceed their max price limit, they're outbid and refunded the unspent portion."
    },
    {
      q: "What happens if an auction doesn't reach its goal?",
      a: 'Every participant gets a full refund, automatically. No locked funds, no rug risk.'
    }
  ]

  let timer: number | null = null
  onMount(() => {
    return toastRun(async (_signal, abortingQue) => {
      const res = await listingActor.get_auction([])
      latestAuction = res[0] || null
      const auctionActor =
        res[0] && 'Icp' in res[0].id ? icAuctionActor(res[0].id.Icp) : null

      if (auctionActor) {
        auctionActor.auction_info().then((info) => {
          auction = info[0] || null
        })
      }

      timer = window.setInterval(() => {
        nowMs = Date.now()
        if (
          isActive() &&
          latestAuction &&
          auctionActor &&
          nowMs < latestAuction.end_time
        ) {
          auctionActor.auction_info().then((info) => {
            auction = info[0] || null
          })
        }
      }, 10_000)

      abortingQue.push(() => {
        if (timer) clearInterval(timer)
        timer == null
      })
    }).abort
  })

  onDestroy(() => {
    if (timer != null) window.clearInterval(timer)
    timer = null
  })
</script>

<div class="flex min-h-screen flex-col">
  <!-- Global decorative elements -->
  <div
    class="pointer-events-none fixed inset-0 overflow-hidden"
    aria-hidden="true"
  >
    <!-- Gradient orbs -->
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

    <!-- Grid pattern overlay -->
    <div class="grid-pattern absolute inset-0"></div>
  </div>

  <!-- Header -->

  <Header>
    <nav
      class="text-muted hidden items-center gap-6 text-sm font-medium md:flex"
    >
      {#each navLinks as link}
        <a
          class="hover:text-foreground relative block p-2 transition after:absolute after:-bottom-1 after:left-0 after:h-0.5 after:w-0 after:bg-current after:transition-all hover:after:w-full"
          href={link.href}
        >
          {link.label}
        </a>
      {/each}
    </nav>
  </Header>

  <main
    class="relative z-10 mx-auto mt-10 max-w-6xl space-y-10 px-4 md:mt-20 md:px-8"
  >
    <!-- Hero Section -->
    <section class="grid gap-12 lg:grid-cols-[1.1fr_0.9fr] lg:items-center">
      <div class="relative space-y-10">
        <div
          class="border-border-subtle bg-surface/50 inline-flex items-center gap-3 rounded-full border px-3 py-1.5 text-xs font-bold tracking-wider uppercase backdrop-blur-sm md:px-5 md:py-2"
        >
          <a
            class="flex flex-row items-center gap-1.5 text-indigo-500 transition-colors hover:text-indigo-400"
            href="https://dashboard.internetcomputer.org/sns/d7wvo-iiaaa-aaaaq-aacsq-cai"
            target="_blank"
            rel="noreferrer"
          >
            <span>Fully Onchain</span>
            <span class="*:size-3.5"><ArrowRightUpLine /></span>
          </a>
          <span class="text-border-subtle">|</span>
          <a
            class="text-muted hover:text-foreground flex flex-row items-center gap-1.5 transition-colors"
            href="https://github.com/ldclabs/token-listing"
            target="_blank"
            rel="noreferrer"
          >
            <span>Open Source</span>
            <span class="*:size-3.5"><ArrowRightUpLine /></span>
          </a>
          <span class="text-border-subtle">|</span>
          <a
            class="flex flex-row items-center gap-1.5 text-amber-600 transition-colors hover:text-amber-500"
            href="https://nns.ic0.app/proposals/?u=d7wvo-iiaaa-aaaaq-aacsq-cai"
            target="_blank"
            rel="noreferrer"
          >
            <span>SNS DAO</span>
            <span class="*:size-3.5"><ArrowRightUpLine /></span>
          </a>
        </div>

        <div class="space-y-6">
          <h1
            class="font-serif text-5xl leading-[1.05] font-bold tracking-tight sm:text-6xl lg:text-7xl"
          >
            Launch tokens <span class="gradient-text">fairly</span>.<br
              class="hidden sm:block"
            />
            Store metadata <span class="gradient-text">forever</span>.
          </h1>
          <p class="text-muted max-w-2xl text-lg leading-relaxed md:text-xl">
            tokenlist.ing is the premier launchpad for sovereign projects. Our <strong
              class="text-foreground font-semibold"
              >Continuous Clearing Auction</strong
            >
            neutralizes bots, while permanent on-chain storage ensures your token's
            legacy across the multi-chain web.
          </p>
        </div>

        <div class="flex flex-wrap items-center gap-5">
          <a
            class="group bg-foreground text-background relative overflow-hidden rounded-full px-8 py-4 text-xs font-bold tracking-widest uppercase transition-all hover:-translate-y-1 hover:shadow-xl active:translate-y-0"
            href="#launchpad"
          >
            <span class="relative z-10">How CCA works</span>
            <div
              class="absolute inset-0 translate-y-full bg-indigo-600 transition-transform duration-300 group-hover:translate-y-0"
            ></div>
          </a>
          <a
            class="border-border-subtle text-muted hover:border-foreground hover:text-foreground hover:bg-surface/50 rounded-full border px-8 py-4 text-xs font-bold tracking-widest uppercase transition-all"
            href="#features"
          >
            All features
          </a>
        </div>
      </div>

      <!-- Live Auction Preview -->
      <div class="relative">
        {#if latest}
          <a
            class="glass-border group bg-surface/30 relative block overflow-hidden rounded-2xl p-1 transition-all duration-500 hover:-translate-y-2 hover:shadow-2xl"
            href={latest.href}
          >
            <div
              class="relative overflow-hidden rounded-[calc(1rem-1px)] p-6 md:p-8"
            >
              <div class="flex items-center justify-between">
                <div
                  class="rounded-full bg-indigo-500/10 px-3 py-1 text-xs font-bold tracking-wider text-indigo-500 uppercase"
                >
                  Latest Auction
                </div>
                <div class="flex items-center gap-2">
                  {#if latest.isActive}
                    <span class="relative flex h-2 w-2">
                      <span
                        class="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75"
                      ></span>
                      <span
                        class="relative inline-flex h-2 w-2 rounded-full bg-emerald-500"
                      ></span>
                    </span>
                  {:else}
                    <span class="bg-border-subtle h-2 w-2 rounded-full"></span>
                  {/if}
                  <span
                    class="text-muted text-xs font-bold tracking-wider uppercase"
                    >{latest.status}</span
                  >
                </div>
              </div>

              <div class="mt-10 flex items-center justify-between gap-4">
                <div class="flex min-w-0 items-center gap-4">
                  <div
                    class="bg-surface ring-border-subtle relative h-14 w-14 shrink-0 overflow-hidden rounded-2xl shadow-inner ring-1"
                  >
                    {#if latest.a.token_logo_url}
                      <img
                        class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-110"
                        src={latest.a.token_logo_url}
                        alt={latest.a.token_symbol}
                        loading="lazy"
                      />
                    {/if}
                  </div>
                  <div class="min-w-0">
                    <h3 class="truncate text-xl font-bold tracking-tight">
                      {latest.a.name || latest.a.token_symbol}
                    </h3>
                    <div
                      class="text-muted mt-1 flex items-center gap-2 text-xs font-medium"
                    >
                      <span class="text-indigo-500"
                        >{latest.a.token_symbol}</span
                      >
                      <span class="text-border-subtle">/</span>
                      <span>{latest.a.currency_symbol}</span>
                      <span class="text-border-subtle">·</span>
                      <span
                        class="bg-surface rounded px-1.5 py-0.5 text-xs uppercase"
                        >{latest.chain}</span
                      >
                    </div>
                  </div>
                </div>
                <div
                  class="bg-surface border-border-subtle flex h-10 w-10 items-center justify-center rounded-full border transition-colors group-hover:border-indigo-500 group-hover:text-indigo-500"
                >
                  <ArrowRightUpLine />
                </div>
              </div>

              <div class="mt-10 space-y-8">
                <div class="space-y-1">
                  <p
                    class="text-muted text-xs font-bold tracking-widest uppercase"
                  >
                    Total Raised
                  </p>
                  <div class="flex items-baseline gap-2">
                    <span
                      class="font-mono text-4xl font-bold tracking-tighter sm:text-5xl"
                      >{latest.raised}</span
                    >
                    <span class="text-muted text-base font-bold uppercase"
                      >{latest.currency.symbol}</span
                    >
                  </div>
                </div>

                <div
                  class="border-border-subtle grid grid-cols-2 gap-x-6 gap-y-8 border-t pt-8"
                >
                  <div class="space-y-1">
                    <p
                      class="text-muted text-xs font-bold tracking-widest uppercase"
                    >
                      Total Committed
                    </p>
                    <div class="flex items-baseline gap-1.5">
                      <span class="truncate font-mono text-xl font-bold"
                        >{latest.totalCommitted}</span
                      >
                      <span class="text-muted text-[10px] font-bold uppercase"
                        >{latest.currency.symbol}</span
                      >
                    </div>
                  </div>
                  <div class="space-y-1">
                    <p
                      class="text-muted text-xs font-bold tracking-widest uppercase"
                    >
                      Current Price
                    </p>
                    <div class="flex items-baseline gap-1.5">
                      <span
                        class="truncate font-mono text-xl font-bold text-indigo-500"
                        >{latest.clearingPriceValue}</span
                      >
                      <span class="text-muted text-[10px] font-bold uppercase"
                        >{latest.currency.symbol}</span
                      >
                    </div>
                  </div>
                  <div class="space-y-1">
                    <p
                      class="text-muted text-xs font-bold tracking-widest uppercase"
                    >
                      Bidders / Bids
                    </p>
                    <p class="truncate font-mono text-xl font-bold">
                      {latest.totalBidders}
                      <span class="text-muted text-sm font-medium">/</span>
                      {latest.bidsCount}
                    </p>
                  </div>
                  <div class="space-y-1">
                    <p
                      class="text-muted text-xs font-bold tracking-widest uppercase"
                    >
                      Goal
                    </p>
                    <p class="truncate font-mono text-xl font-bold">
                      {latest.requiredCurrency}
                      <span class="text-muted text-[10px] font-bold uppercase"
                        >{latest.currency.symbol}</span
                      >
                    </p>
                  </div>
                </div>

                <div class="bg-surface/50 rounded-2xl p-5">
                  <div class="flex items-center justify-between">
                    <p
                      class="text-muted text-xs font-bold tracking-widest uppercase"
                    >
                      {#if latest.status === 'Upcoming'}
                        Starts In
                      {:else}
                        Time Remaining
                      {/if}
                    </p>
                    <p class="font-mono text-sm font-bold text-indigo-500">
                      {#if latest.status === 'Ended'}
                        Auction Ended
                      {:else}
                        {latest.remaining}
                      {/if}
                    </p>
                  </div>
                  <div
                    class="mt-3 flex h-1.5 overflow-hidden rounded-full bg-black/10 dark:bg-white/10"
                  >
                    <div
                      class="bg-linear-to-r from-indigo-500 to-purple-500 transition-all duration-1000"
                      style="width: 65%"
                    ></div>
                  </div>
                  <p class="text-muted mt-3 text-xs font-medium">
                    {formatDatetime(latest.a.start_time)} → {formatDatetime(
                      latest.a.end_time
                    )}
                  </p>
                </div>
              </div>

              <!-- Decorative glow -->
              <div
                class="animate-pulse-glow absolute -right-20 -bottom-20 h-64 w-64 rounded-full bg-indigo-500/10 blur-3xl"
                aria-hidden="true"
              ></div>
            </div></a
          >
        {:else}
          <div
            class="glass-border bg-surface/30 relative animate-pulse overflow-hidden rounded-2xl p-8"
          >
            <div class="bg-surface mb-8 h-4 w-24 rounded"></div>
            <div class="mb-10 flex items-center gap-4">
              <div class="bg-surface h-14 w-14 rounded-2xl"></div>
              <div class="space-y-2">
                <div class="bg-surface h-6 w-32 rounded"></div>
                <div class="bg-surface h-4 w-24 rounded"></div>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-8">
              <div class="space-y-2">
                <div class="bg-surface h-3 w-16 rounded"></div>
                <div class="bg-surface h-8 w-24 rounded"></div>
              </div>
              <div class="space-y-2">
                <div class="bg-surface h-3 w-16 rounded"></div>
                <div class="bg-surface h-8 w-24 rounded"></div>
              </div>
            </div>
          </div>
        {/if}
      </div>
    </section>

    <!-- Features Section -->
    <section id="features" class="space-y-12 pt-20 md:pt-32">
      <div
        class="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between"
      >
        <div class="space-y-2">
          <p
            class="text-xs font-bold tracking-[0.2em] text-indigo-500 uppercase"
          >
            Platform Pillars
          </p>
          <h2 class="font-serif text-4xl font-bold tracking-tight sm:text-5xl">
            Everything tokens need
          </h2>
        </div>
        <a
          class="text-muted hover:text-foreground text-xs font-bold tracking-wider uppercase underline underline-offset-8 transition-colors"
          href="#cta"
        >
          Request access →
        </a>
      </div>

      <div class="grid gap-6 md:grid-cols-2">
        {#each pillars as item, i (i)}
          <div
            class="glass-border group bg-surface/30 hover:bg-surface/50 relative overflow-hidden rounded-2xl p-8 transition-all duration-500 hover:-translate-y-1.5 hover:shadow-2xl"
          >
            <div
              class="absolute -top-12 -right-12 h-40 w-40 rounded-full bg-indigo-500/5 blur-3xl transition-colors duration-500 group-hover:bg-indigo-500/10"
            ></div>

            <div class="relative space-y-6">
              <div class="flex items-center justify-between">
                <div
                  class="bg-surface border-border-subtle flex h-12 w-12 items-center justify-center rounded-xl border text-2xl shadow-sm transition-transform duration-500 group-hover:scale-110"
                >
                  {item.icon}
                </div>
                <div
                  class="text-muted border-border-subtle rounded-full border px-3 py-1 text-xs font-bold tracking-widest uppercase"
                >
                  {item.tag}
                </div>
              </div>
              <div class="space-y-3">
                <h3 class="text-2xl font-bold tracking-tight">{item.title}</h3>
                <p class="text-muted text-base leading-relaxed">
                  {item.desc}
                </p>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </section>

    <!-- CCA Launchpad Section -->
    <section id="launchpad" class="pt-20 md:pt-32">
      <div
        class="glass-border bg-surface/20 relative space-y-16 overflow-hidden rounded-3xl p-8 md:p-16"
      >
        <!-- Decorative background -->
        <div class="pointer-events-none absolute inset-0" aria-hidden="true">
          <div
            class="animate-float-slow absolute -top-40 -right-40 h-96 w-96 rounded-full bg-indigo-500/5 blur-[100px]"
          ></div>
          <div
            class="animate-float absolute -bottom-40 -left-40 h-96 w-96 rounded-full bg-amber-500/5 blur-[100px]"
          ></div>
        </div>

        <div
          class="relative flex flex-col gap-8 lg:flex-row lg:items-end lg:justify-between"
        >
          <div class="max-w-3xl space-y-6">
            <p
              class="text-xs font-bold tracking-[0.2em] text-amber-500 uppercase"
            >
              Continuous Clearing Auction
            </p>
            <h2
              class="font-serif text-4xl font-bold tracking-tight sm:text-5xl lg:text-6xl"
            >
              A <span class="gradient-text">better</span> way to launch
            </h2>
            <p class="text-muted text-lg leading-relaxed md:text-xl">
              Traditional token sales are a race—bots win, regular users lose.
              CCA flips the script by spreading every bid across the entire
              auction. The result? <strong class="text-foreground font-semibold"
                >Fair prices determined by real demand, not transaction timing.</strong
              >
            </p>
          </div>
          <a
            class="text-muted hover:text-foreground inline-flex items-center gap-2 text-xs font-bold tracking-wider uppercase underline underline-offset-8 transition-colors"
            href="https://github.com/ldclabs/token-listing/blob/main/docs/cca.md"
          >
            Read the whitepaper ↗
          </a>
        </div>

        <div class="relative grid gap-6 sm:grid-cols-2 lg:grid-cols-5">
          {#each steps as step, i}
            <div
              class="glass-border group bg-surface/40 hover:bg-surface/60 relative flex flex-col gap-5 rounded-2xl p-6 transition-all duration-500 hover:-translate-y-2 hover:shadow-xl"
            >
              <div
                class="flex h-10 w-10 items-center justify-center rounded-xl bg-linear-to-br from-indigo-500 to-purple-600 text-sm font-bold text-white shadow-lg transition-transform duration-500 group-hover:scale-110 group-hover:rotate-3"
              >
                {i + 1}
              </div>
              <div class="space-y-2">
                <h3 class="text-lg font-bold tracking-tight">{step.title}</h3>
                <p class="text-muted text-sm leading-relaxed">{step.body}</p>
              </div>

              {#if i < steps.length - 1}
                <div
                  class="bg-border-subtle absolute top-1/2 -right-3 hidden h-px w-6 lg:block"
                ></div>
              {/if}
            </div>
          {/each}
        </div>

        <div class="relative grid gap-8 md:grid-cols-2">
          <div
            class="glass-border group rounded-2xl bg-indigo-500/5 p-8 transition-all duration-500 hover:bg-indigo-500/10"
          >
            <div class="flex items-center gap-4">
              <div
                class="flex h-10 w-10 items-center justify-center rounded-full bg-indigo-500/20 text-xl"
              >
                🚀
              </div>
              <p
                class="text-xs font-bold tracking-widest text-indigo-500 uppercase"
              >
                For Projects
              </p>
            </div>
            <ul class="mt-8 space-y-4">
              {#each [['Predictable outcomes', 'know your minimum raise before you start'], ['Instant liquidity', 'auction proceeds seed your trading pool automatically'], ['Credible distribution', "prove to your community that insiders didn't front-run"]] as [title, desc]}
                <li class="flex items-start gap-3">
                  <span class="mt-1 text-indigo-500">
                    <svg
                      class="h-4 w-4"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="3"
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                  </span>
                  <p class="text-muted text-sm leading-relaxed">
                    <strong class="text-foreground font-semibold"
                      >{title}</strong
                    >
                    — {desc}
                  </p>
                </li>
              {/each}
            </ul>
          </div>
          <div
            class="glass-border group rounded-2xl bg-amber-500/5 p-8 transition-all duration-500 hover:bg-amber-500/10"
          >
            <div class="flex items-center gap-4">
              <div
                class="flex h-10 w-10 items-center justify-center rounded-full bg-amber-500/20 text-xl"
              >
                👥
              </div>
              <p
                class="text-xs font-bold tracking-widest text-amber-500 uppercase"
              >
                For Participants
              </p>
            </div>
            <ul class="mt-8 space-y-4">
              {#each [['No timing pressure', 'bid early, bid late, same fair treatment'], ['Set your max price', 'if clearing exceeds your limit, you get refunded'], ['Full refund guarantee', 'failed auctions return 100% of funds']] as [title, desc]}
                <li class="flex items-start gap-3">
                  <span class="mt-1 text-amber-500">
                    <svg
                      class="h-4 w-4"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="3"
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                  </span>
                  <p class="text-muted text-sm leading-relaxed">
                    <strong class="text-foreground font-semibold"
                      >{title}</strong
                    >
                    — {desc}
                  </p>
                </li>
              {/each}
            </ul>
          </div>
        </div>
      </div>
    </section>

    <!-- Chains Section -->
    <section id="chains" class="space-y-12 pt-20 md:pt-32">
      <div
        class="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between"
      >
        <div class="space-y-2">
          <p
            class="text-xs font-bold tracking-[0.2em] text-indigo-500 uppercase"
          >
            Multi-chain
          </p>
          <h2 class="font-serif text-4xl font-bold tracking-tight sm:text-5xl">
            One platform, <span class="gradient-text">every chain</span>
          </h2>
        </div>
        <p class="text-muted text-sm font-medium">
          Powered by ICP's Chain-Key & HTTPs-Outcall capabilities
        </p>
      </div>

      <div class="grid gap-4 md:grid-cols-4">
        {#each chains as chain}
          <div
            class="glass-border group bg-surface/30 hover:bg-surface/50 relative overflow-hidden rounded-2xl p-6 transition-all duration-500 hover:-translate-y-1.5 hover:shadow-xl"
          >
            <!-- Gradient accent bar -->
            <div
              class="absolute top-0 right-0 left-0 h-1 bg-linear-to-r {chain.color} opacity-20 transition-opacity duration-500 group-hover:opacity-100"
            ></div>

            <div class="flex items-center justify-between">
              <span class="text-lg font-bold tracking-tight">{chain.name}</span>
              <div
                class="text-muted transition-transform duration-500 group-hover:translate-x-1 group-hover:text-indigo-500"
              >
                <ArrowRightUpLine />
              </div>
            </div>
            <p class="text-muted mt-3 text-xs leading-relaxed font-medium"
              >{chain.note}</p
            >
          </div>
        {/each}
      </div>

      <div
        class="glass-border bg-surface/20 relative overflow-hidden rounded-3xl p-8 md:p-12"
      >
        <div class="relative grid gap-12 md:grid-cols-2">
          <div class="space-y-4">
            <div class="flex items-center gap-3">
              <div
                class="flex h-8 w-8 items-center justify-center rounded-lg bg-indigo-500/10 text-indigo-500"
              >
                ⚙️
              </div>
              <p
                class="text-xs font-bold tracking-widest text-indigo-500 uppercase"
              >
                How it works
              </p>
            </div>
            <p class="text-muted text-base leading-relaxed">
              ICP acts as the coordination layer—storing metadata, running
              auctions, and managing cross-chain state. Your tokens exist
              natively on their target chains, with ICP ensuring everything
              stays in sync.
            </p>
          </div>
          <div class="space-y-4">
            <div class="flex items-center gap-3">
              <div
                class="flex h-8 w-8 items-center justify-center rounded-lg bg-amber-500/10 text-amber-500"
              >
                🛠️
              </div>
              <p
                class="text-xs font-bold tracking-widest text-amber-500 uppercase"
              >
                Developer tools
              </p>
            </div>
            <p class="text-muted text-base leading-relaxed">
              SDKs for TypeScript, Rust, and Motoko. REST APIs for metadata
              queries. Webhooks for auction events. Build on tokenlist.ing
              without learning new paradigms.
            </p>
          </div>
        </div>
      </div>
    </section>

    <!-- FAQ Section -->
    <section id="faq" class="pt-20 md:pt-32">
      <div
        class="glass-border bg-surface/20 space-y-12 rounded-3xl p-8 md:p-16"
      >
        <div
          class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between"
        >
          <h2 class="font-serif text-4xl font-bold tracking-tight sm:text-5xl">
            Frequently asked questions
          </h2>
          <a
            class="text-muted hover:text-foreground text-xs font-bold tracking-wider uppercase underline underline-offset-8 transition-colors"
            href="mailto:hello@tokenlist.ing"
          >
            More questions?
          </a>
        </div>
        <div class="grid gap-4">
          {#each faqs as item, i (i)}
            <div
              class="glass-border group bg-surface/40 hover:bg-surface/60 rounded-2xl p-6 transition-all duration-500"
            >
              <div class="flex items-start gap-4">
                <span
                  class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-linear-to-br from-indigo-500 to-purple-600 text-xs font-bold text-white shadow-md"
                  >{i + 1}</span
                >
                <div class="space-y-3">
                  <h3 class="text-foreground text-lg font-bold tracking-tight">
                    {item.q}
                  </h3>
                  <p class="text-muted text-base leading-relaxed">{item.a}</p>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </section>

    <!-- CTA Section -->
    <section
      id="cta"
      class="bg-foreground relative mt-20 overflow-hidden rounded-3xl px-8 py-20 text-center md:mt-32 md:py-32"
    >
      <!-- Animated background -->
      <div
        class="pointer-events-none absolute inset-0 opacity-20"
        aria-hidden="true"
      >
        <div
          class="animate-float-slow absolute -top-20 -left-20 h-96 w-96 rounded-full bg-indigo-400 blur-[100px]"
        ></div>
        <div
          class="animate-float absolute -right-20 -bottom-20 h-96 w-96 rounded-full bg-amber-400 blur-[100px]"
        ></div>
      </div>

      <div class="relative z-10 mx-auto max-w-3xl space-y-10">
        <div class="space-y-4">
          <p
            class="text-xs font-bold tracking-[0.3em] text-indigo-400 uppercase"
          >
            Early Access
          </p>
          <h2
            class="text-background font-serif text-4xl font-bold tracking-tight sm:text-5xl lg:text-6xl"
          >
            Ready to launch <span class="text-indigo-400">fairly</span>?
          </h2>
          <p
            class="text-background/70 mx-auto max-w-2xl text-lg leading-relaxed"
          >
            We're onboarding the first wave of projects now. Tell us about your
            token, your community, and your timeline—we'll help you configure
            the perfect launch.
          </p>
        </div>
        <div class="flex flex-wrap justify-center gap-6">
          <a
            class="group relative overflow-hidden rounded-full bg-indigo-500 px-10 py-5 text-xs font-bold tracking-widest text-white uppercase transition-all hover:-translate-y-1 hover:bg-indigo-600 hover:shadow-2xl active:translate-y-0"
            href="https://x.com/ICPandaDAO"
          >
            <span class="relative z-10">Get in touch</span>
          </a>
          <a
            class="border-background/20 text-background hover:border-background hover:bg-background/5 rounded-full border px-10 py-5 text-xs font-bold tracking-widest uppercase transition-all"
            href="https://github.com/ldclabs/token-listing/blob/main/docs/cca.md"
          >
            Read the docs
          </a>
        </div>
      </div>
    </section>

    <!-- Footer -->
    <footer class="border-border-subtle border-t py-10 md:py-20">
      <div class="flex flex-col items-center">
        <p class="flex flex-row items-center gap-1">
          <span class="text-sm">© {new Date().getFullYear()}</span>
          <a
            class="transition-transform hover:scale-105"
            href="https://panda.fans"
            target="_blank"
            ><img
              class="w-28"
              src={theme === 'dark'
                ? '/_assets/icpanda-dao-white.svg'
                : '/_assets/icpanda-dao.svg'}
              alt="ICPanda DAO"
            /></a
          >
        </p>
        <p class="mt-2 text-center text-sm antialiased">
          Breathing life into sovereign AI.<br />We are building the open-source
          stack for agents to remember, transact, and evolve as first-class
          citizens in Web3.<br />
          <a
            class="underline underline-offset-4 transition-colors hover:text-amber-500"
            href="https://anda.ai"
            target="_blank">Anda.AI</a
          >
          <span class="mx-1">|</span>
          <a
            class="underline underline-offset-4 transition-colors hover:text-amber-500"
            href="https://dmsg.net">dMsg.net</a
          >
          <span class="mx-1">|</span>
          <a
            class="underline underline-offset-4 transition-colors hover:text-amber-500"
            href="https://1bridge.app/">1Bridge.app</a
          >
        </p>
      </div>
    </footer>
  </main>
</div>
