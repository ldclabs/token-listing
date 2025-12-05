<script lang="ts">
  import ArrowRightUpLine from '$lib/icons/arrow-right-up-line.svelte'
  import { onMount } from 'svelte'

  type Theme = 'light' | 'dark'
  let theme = $state<Theme>('dark')

  const setTheme = (value: Theme) => {
    theme = value
    localStorage['theme'] = value

    document.documentElement.classList.toggle(
      'dark',
      localStorage['theme'] === 'dark' ||
        (!('theme' in localStorage) &&
          window.matchMedia('(prefers-color-scheme: dark)').matches)
    )
  }

  const toggleTheme = () => setTheme(theme === 'dark' ? 'light' : 'dark')

  onMount(() => {
    setTheme(localStorage['theme'] || 'dark')
  })

  const navLinks = [
    { label: 'Features', href: '#features' },
    { label: 'Launchpad', href: '#launchpad' },
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
</script>

<div class="relative flex min-h-screen flex-col overflow-x-hidden">
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
  <header
    class="border-border-subtle sticky top-0 z-50 border-b shadow-sm backdrop-blur-xl"
  >
    <div
      class="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-4 px-4 py-2 md:px-8"
    >
      <div class="flex items-center gap-3">
        <a href="/" class="bg-white p-0.5"
          ><img
            alt="TokenList.ing Logo"
            src="/_assets/logo.webp"
            class="h-12 w-12 rounded-sm object-contain transition-transform hover:scale-105"
          /></a
        >
        <div>
          <p class="text-muted font-serif text-lg tracking-[0.25em]"
            >TokenList.ing</p
          >
          <p class="text-foreground hidden text-base font-medium md:flex">
            Token infrastructure for the multi-chain era
          </p>
        </div>
      </div>

      <nav
        class="text-muted hidden items-center gap-6 text-sm font-medium md:flex"
      >
        {#each navLinks as link}
          <a
            class="hover:text-foreground relative transition after:absolute after:-bottom-1 after:left-0 after:h-0.5 after:w-0 after:bg-current after:transition-all hover:after:w-full"
            href={link.href}
          >
            {link.label}
          </a>
        {/each}
        <a
          class="border-border-subtle hover:border-foreground hover:bg-accent hover:text-accent-foreground rounded-full border px-4 py-2 text-sm font-semibold transition-all hover:scale-105"
          href="#cta"
        >
          Get early access
        </a>
      </nav>

      <button
        class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-4 py-1 font-semibold tracking-wide uppercase transition-all hover:scale-105"
        onclick={toggleTheme}
        aria-pressed={theme === 'light'}
        aria-label="Toggle theme"
      >
        {theme === 'dark' ? '☀️' : '🌙'}
      </button>
    </div>
  </header>

  <main
    class="relative z-10 mx-auto mt-10 max-w-6xl space-y-10 px-4 md:mt-20 md:px-8"
  >
    <!-- Hero Section -->
    <section class="grid gap-12 lg:grid-cols-[1.1fr_0.9fr]">
      <div class="space-y-8">
        <div
          class="border-border-subtle text-muted bg-surface inline-flex items-center gap-3 rounded-full border px-2 py-1 text-xs font-semibold md:px-4 md:py-2"
        >
          <a
            class="hover:text-foreground flex flex-row items-center gap-1 transition"
            href="https://dashboard.internetcomputer.org/sns/d7wvo-iiaaa-aaaaq-aacsq-cai"
            target="_blank"
            rel="noreferrer"
          >
            <span class="">Fully Onchain</span>
            <span class="*:size-4"><ArrowRightUpLine /></span>
          </a>
          <span class="">|</span>
          <a
            class="hover:text-foreground flex flex-row items-center gap-1 transition"
            href="https://github.com/ldclabs/token-listing"
            target="_blank"
            rel="noreferrer"
          >
            <span class="">Open Source</span>
            <span class="*:size-4"><ArrowRightUpLine /></span>
          </a>
          <span class="">|</span>
          <a
            class="hover:text-foreground flex flex-row items-center gap-1 transition"
            href="https://nns.ic0.app/proposals/?u=d7wvo-iiaaa-aaaaq-aacsq-cai"
            target="_blank"
            rel="noreferrer"
          >
            <span class="">SNS DAO Gov</span>
            <span class="*:size-4"><ArrowRightUpLine /></span>
          </a>
        </div>

        <div class="space-y-6">
          <h1
            class="font-serif text-4xl leading-[1.1] font-semibold sm:text-5xl lg:text-6xl"
          >
            Launch tokens <span class="gradient-text">fairly</span>.<br
              class="hidden sm:block"
            />
            Store metadata <span class="gradient-text">forever</span>.
          </h1>
          <p class="text-muted max-w-2xl text-lg leading-relaxed">
            tokenlist.ing is where projects come to launch. Our <strong
              class="text-foreground">Continuous Clearing Auction</strong
            >
            eliminates bots and sniping, while permanent on-chain storage keeps your
            token's identity alive across every chain.
          </p>
        </div>

        <div class="flex flex-wrap items-center gap-4">
          <a
            class="group bg-accent text-accent-foreground relative overflow-hidden rounded-full px-6 py-3 text-sm font-semibold tracking-wide uppercase transition-all hover:-translate-y-0.5 hover:shadow-lg"
            href="#launchpad"
          >
            <span class="relative z-10">How CCA works</span>
            <div class="shimmer absolute inset-0"></div>
          </a>
          <a
            class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-6 py-3 text-sm font-semibold transition-all hover:scale-105"
            href="#features"
          >
            All features
          </a>
        </div>
      </div>

      <!-- Live Auction Preview -->
      <div class="relative">
        <div
          class="animated-border glass-border from-surface-hover relative overflow-hidden rounded-xl bg-linear-to-b via-transparent to-transparent p-4 md:p-8"
        >
          <div class="text-muted flex items-center justify-between text-sm">
            <span>Live auction preview</span>
            <span class="flex items-center gap-1.5">
              <span class="relative flex h-2 w-2">
                <span
                  class="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75"
                ></span>
                <span
                  class="relative inline-flex h-2 w-2 rounded-full bg-green-500"
                ></span>
              </span>
              Active (mock)
            </span>
          </div>

          <div class="mt-8 space-y-6">
            <div>
              <p
                class="text-muted-foreground text-xs tracking-widest uppercase"
              >
                Current price
              </p>
              <p class="font-mono text-3xl font-medium">
                <span class="gradient-text">0.0847</span> USDC
              </p>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div>
                <p
                  class="text-muted-foreground text-xs tracking-widest uppercase"
                >
                  Total raised
                </p>
                <p class="text-muted font-mono text-xl">$127,450</p>
              </div>
              <div>
                <p
                  class="text-muted-foreground text-xs tracking-widest uppercase"
                >
                  Participants
                </p>
                <p class="text-muted font-mono text-xl">342</p>
              </div>
            </div>

            <div class="bg-border-subtle h-px w-full"></div>

            <div>
              <p
                class="text-muted-foreground text-xs tracking-widest uppercase"
              >
                Time remaining
              </p>
              <p class="text-muted font-mono text-xl">2d 14h 22m</p>
            </div>

            <!-- Mini chart visualization -->
            <div class="bg-surface rounded-xl p-4">
              <div class="mb-2 flex h-16 items-end justify-between gap-1">
                {#each [30, 45, 35, 60, 55, 70, 65, 80, 75, 90, 85, 100] as height}
                  <div
                    class="flex-1 rounded-t bg-linear-to-t from-purple-500/40 to-amber-500/80 transition-all hover:from-purple-500/60 hover:to-amber-400"
                    style="height: {height}%;"
                  ></div>
                {/each}
              </div>
              <p class="text-muted-foreground text-xs">
                Price adjusts continuously as bids come in. Your final cost is
                the clearing price at auction end—same for everyone.
              </p>
            </div>
          </div>

          <div
            class="animate-pulse-glow absolute -right-10 -bottom-10 h-40 w-40 rounded-full bg-amber-500/20 blur-2xl"
            aria-hidden="true"
          ></div>
        </div>
      </div>
    </section>

    <!-- Features Section -->
    <section id="features" class="space-y-8 pt-10 md:pt-20">
      <div
        class="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between"
      >
        <div>
          <p class="text-muted-foreground text-sm tracking-widest uppercase">
            Platform
          </p>
          <h2 class="mt-2 font-serif text-3xl font-semibold sm:text-4xl">
            Everything tokens need
          </h2>
        </div>
        <a
          class="text-muted hover:text-foreground text-sm font-semibold underline underline-offset-4"
          href="#cta"
        >
          Request access →
        </a>
      </div>

      <div class="stagger-fade-in grid gap-5 md:grid-cols-2">
        {#each pillars as item, i (i)}
          <div
            class="glass-border group relative overflow-hidden rounded-xl p-4 transition-all duration-300 hover:-translate-y-1 hover:shadow-lg md:p-8"
          >
            <div
              class="bg-surface-hover absolute top-0 right-0 h-24 w-24 translate-x-8 -translate-y-4 rounded-full blur-2xl transition-all duration-500 group-hover:bg-amber-500/10"
            ></div>

            <!-- Icon -->
            <div
              class="absolute top-4 right-4 text-4xl opacity-20 transition-all duration-300 group-hover:scale-110 group-hover:opacity-40 md:top-8 md:right-8"
            >
              {item.icon}
            </div>

            <div
              class="border-border-subtle bg-surface text-muted mb-4 inline-flex items-center rounded-full border px-3 py-1 text-[10px] font-bold tracking-widest uppercase transition-colors group-hover:border-amber-500/50 group-hover:text-amber-400"
            >
              {item.tag}
            </div>
            <h3 class="text-xl font-semibold">{item.title}</h3>
            <p class="text-muted mt-3 text-sm leading-relaxed">
              {item.desc}
            </p>
          </div>
        {/each}
      </div>
    </section>

    <!-- CCA Launchpad Section -->
    <section id="launchpad" class="pt-10 md:pt-20">
      <div
        class="border-border-subtle bg-surface relative space-y-10 overflow-hidden rounded-xl border p-4 md:p-8"
      >
        <!-- Decorative background -->
        <div class="pointer-events-none absolute inset-0" aria-hidden="true">
          <div
            class="animate-float-slow absolute -top-20 -right-20 h-64 w-64 rounded-full bg-purple-500/5 blur-3xl"
          ></div>
          <div
            class="animate-float absolute -bottom-20 -left-20 h-64 w-64 rounded-full bg-amber-500/5 blur-3xl"
          ></div>
        </div>

        <div
          class="relative flex flex-col gap-4 md:flex-row md:items-end md:justify-between"
        >
          <div>
            <p class="text-muted text-sm tracking-widest uppercase">
              Continuous Clearing Auction
            </p>
            <h2 class="mt-2 font-serif text-3xl font-semibold sm:text-4xl">
              A <span class="gradient-text">better</span> way to launch
            </h2>
            <p class="text-muted mt-4 max-w-2xl">
              Traditional token sales are a race—bots win, regular users lose.
              CCA flips the script by spreading every bid across the entire
              auction. The result? <strong class="text-foreground"
                >Fair prices determined by real demand, not transaction timing.</strong
              >
            </p>
          </div>
          <a
            class="text-muted hover:text-foreground inline-flex items-center gap-2 text-sm font-semibold underline underline-offset-4"
            href="https://github.com/ldclabs/token-listing/blob/main/docs/cca.md"
          >
            Read the whitepaper ↗
          </a>
        </div>

        <div
          class="stagger-fade-in relative grid gap-4 sm:grid-cols-2 lg:grid-cols-5"
        >
          {#each steps as step, i}
            <div
              class="glass-border group flex flex-col gap-3 rounded-xl p-4 transition-all duration-300 hover:-translate-y-1 hover:shadow-md"
            >
              <div
                class="flex h-8 w-8 items-center justify-center rounded-full bg-linear-to-br from-purple-500 to-amber-500 text-xs font-bold text-white shadow-lg transition-transform group-hover:scale-110"
              >
                {i + 1}
              </div>
              <h3 class="font-semibold">{step.title}</h3>
              <p class="text-muted text-xs leading-relaxed">{step.body}</p>

              <!-- Connector line (except last item) -->
              {#if i < steps.length - 1}
                <div
                  class="absolute top-1/2 right-0 hidden h-px w-4 bg-linear-to-r from-amber-500/50 to-transparent lg:block"
                  style="transform: translateX(100%);"
                ></div>
              {/if}
            </div>
          {/each}
        </div>

        <div class="relative grid gap-6 md:grid-cols-2">
          <div
            class="glass-border rounded-xl p-4 transition-all duration-300 hover:shadow-lg"
          >
            <p
              class="text-muted flex items-center gap-2 text-sm tracking-widest uppercase"
            >
              <span class="text-xl">🚀</span>
              For projects
            </p>
            <ul class="text-muted mt-4 space-y-3 text-sm">
              <li class="flex items-start gap-2">
                <span class="text-green-500">✓</span>
                <span
                  ><strong class="text-foreground">Predictable outcomes</strong> —
                  know your minimum raise before you start</span
                >
              </li>
              <li class="flex items-start gap-2">
                <span class="text-green-500">✓</span>
                <span
                  ><strong class="text-foreground">Instant liquidity</strong> — auction
                  proceeds seed your trading pool automatically</span
                >
              </li>
              <li class="flex items-start gap-2">
                <span class="text-green-500">✓</span>
                <span
                  ><strong class="text-foreground">Credible distribution</strong
                  > — prove to your community that insiders didn't front-run</span
                >
              </li>
            </ul>
          </div>
          <div
            class="glass-border rounded-xl p-4 transition-all duration-300 hover:shadow-lg"
          >
            <p
              class="text-muted flex items-center gap-2 text-sm tracking-widest uppercase"
            >
              <span class="text-xl">👥</span>
              For participants
            </p>
            <ul class="text-muted mt-4 space-y-3 text-sm">
              <li class="flex items-start gap-2">
                <span class="text-green-500">✓</span>
                <span
                  ><strong class="text-foreground">No timing pressure</strong> — bid
                  early, bid late, same fair treatment</span
                >
              </li>
              <li class="flex items-start gap-2">
                <span class="text-green-500">✓</span>
                <span
                  ><strong class="text-foreground">Set your max price</strong> — if
                  clearing exceeds your limit, you get refunded</span
                >
              </li>
              <li class="flex items-start gap-2">
                <span class="text-green-500">✓</span>
                <span
                  ><strong class="text-foreground">Full refund guarantee</strong
                  > — failed auctions return 100% of funds</span
                >
              </li>
            </ul>
          </div>
        </div>
      </div>
    </section>

    <!-- Chains Section -->
    <section id="chains" class="space-y-8 pt-10 md:pt-20">
      <div
        class="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between"
      >
        <div>
          <p class="text-muted-foreground text-sm tracking-widest uppercase">
            Multi-chain
          </p>
          <h2 class="mt-2 font-serif text-3xl font-semibold sm:text-4xl">
            One platform, <span class="gradient-text">every chain</span>
          </h2>
        </div>
        <p class="text-muted text-sm">
          Powered by ICP's Chain-Key & HTTPs-Outcall capabilities
        </p>
      </div>

      <div class="stagger-fade-in grid gap-4 md:grid-cols-4">
        {#each chains as chain}
          <div
            class="glass-border group overflow-hidden rounded-xl p-4 transition-all duration-300 hover:-translate-y-1 hover:shadow-lg"
          >
            <!-- Gradient accent bar -->
            <div
              class="absolute top-0 right-0 left-0 h-1 bg-linear-to-r {chain.color} opacity-0 transition-opacity group-hover:opacity-100"
            ></div>

            <div class="flex items-center justify-between">
              <span class="font-semibold">{chain.name}</span>
              <span
                class="text-muted-foreground transition-transform group-hover:translate-x-1"
                >→</span
              >
            </div>
            <p class="text-muted mt-2 text-xs">{chain.note}</p>
          </div>
        {/each}
      </div>

      <div class="glass-border relative overflow-hidden rounded-xl p-4 md:p-8">
        <!-- Chain connection visualization -->
        <div
          class="pointer-events-none absolute inset-0 flex items-center justify-center opacity-5"
          aria-hidden="true"
        >
          <svg class="h-full w-full" viewBox="0 0 400 200">
            <circle
              cx="100"
              cy="100"
              r="60"
              stroke="currentColor"
              fill="none"
              stroke-width="1"
            />
            <circle
              cx="200"
              cy="100"
              r="80"
              stroke="currentColor"
              fill="none"
              stroke-width="1"
            />
            <circle
              cx="300"
              cy="100"
              r="60"
              stroke="currentColor"
              fill="none"
              stroke-width="1"
            />
            <line
              x1="160"
              y1="100"
              x2="120"
              y2="100"
              stroke="currentColor"
              stroke-width="1"
            />
            <line
              x1="280"
              y1="100"
              x2="240"
              y2="100"
              stroke="currentColor"
              stroke-width="1"
            />
          </svg>
        </div>

        <div class="relative grid gap-6 md:grid-cols-2">
          <div>
            <p
              class="text-muted flex items-center gap-2 text-sm tracking-widest uppercase"
            >
              <span class="text-xl">⚙️</span>
              How it works
            </p>
            <p class="text-muted mt-2 text-sm">
              ICP acts as the coordination layer—storing metadata, running
              auctions, and managing cross-chain state. Your tokens exist
              natively on their target chains, with ICP ensuring everything
              stays in sync.
            </p>
          </div>
          <div>
            <p
              class="text-muted flex items-center gap-2 text-sm tracking-widest uppercase"
            >
              <span class="text-xl">🛠️</span>
              Developer tools
            </p>
            <p class="text-muted mt-2 text-sm">
              SDKs for TypeScript, Rust, and Motoko. REST APIs for metadata
              queries. Webhooks for auction events. Build on tokenlist.ing
              without learning new paradigms.
            </p>
          </div>
        </div>
      </div>
    </section>

    <!-- FAQ Section -->
    <section id="faq" class="pt-10 md:pt-20">
      <div
        class="border-border-subtle bg-surface space-y-6 rounded-xl border p-4 md:p-8"
      >
        <div class="flex items-center justify-between">
          <h2 class="font-serif text-2xl font-semibold sm:text-3xl">
            Frequently asked questions
          </h2>
          <a
            class="text-muted hover:text-foreground text-sm underline underline-offset-4"
            href="mailto:hello@tokenlist.ing"
          >
            More questions?
          </a>
        </div>
        <div class="stagger-fade-in space-y-4">
          {#each faqs as item, i (i)}
            <div
              class="glass-border group rounded-xl p-4 transition-all duration-300 hover:-translate-y-1 hover:shadow-md"
            >
              <p class="text-foreground flex items-start gap-3 font-semibold">
                <span
                  class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-linear-to-br from-purple-500 to-amber-500 text-xs text-white"
                  >{i + 1}</span
                >
                {item.q}
              </p>
              <p class="text-muted mt-2 pl-9 text-sm leading-relaxed"
                >{item.a}</p
              >
            </div>
          {/each}
        </div>
      </div>
    </section>

    <!-- CTA Section -->
    <section
      id="cta"
      class="animated-border relative overflow-hidden rounded-xl p-4 pt-10 text-center md:p-8 md:pt-20"
    >
      <!-- Animated background -->
      <div class="pointer-events-none absolute inset-0" aria-hidden="true">
        <div
          class="animate-float-slow absolute top-10 left-10 h-32 w-32 rounded-full bg-purple-500/10 blur-2xl"
        ></div>
        <div
          class="animate-float absolute right-10 bottom-10 h-40 w-40 rounded-full bg-amber-500/10 blur-2xl"
        ></div>
        <div
          class="animate-float-reverse absolute top-1/2 left-1/2 h-24 w-24 -translate-x-1/2 -translate-y-1/2 rounded-full bg-yellow-500/10 blur-2xl"
        ></div>
      </div>

      <div class="relative">
        <p class="text-muted text-sm tracking-widest uppercase">
          Early access
        </p>
        <h2 class="mt-3 font-serif text-3xl font-semibold sm:text-4xl">
          Ready to launch <span class="gradient-text">fairly</span>?
        </h2>
        <p class="text-muted mx-auto mt-4 max-w-2xl text-sm">
          We're onboarding the first wave of projects now. Tell us about your
          token, your community, and your timeline—we'll help you configure the
          perfect launch.
        </p>
        <div class="mt-8 flex flex-wrap justify-center gap-4">
          <a
            class="group bg-accent text-accent-foreground relative overflow-hidden rounded-full px-6 py-3 text-sm font-semibold tracking-wide uppercase transition-all hover:-translate-y-0.5 hover:shadow-lg"
            href="https://x.com/ICPandaDAO"
          >
            <span class="relative z-10">Get in touch</span>
            <div class="shimmer absolute inset-0"></div>
          </a>
          <a
            class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-6 py-3 text-sm font-semibold transition-all hover:scale-105"
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
            class="underline underline-offset-4 transition-colors hover:text-amber-400"
            href="https://anda.ai"
            target="_blank">Anda.AI</a
          >
          <span class="mx-1">|</span>
          <a
            class="underline underline-offset-4 transition-colors hover:text-amber-400"
            href="https://dmsg.net">dMsg.net</a
          >
          <span class="mx-1">|</span>
          <a
            class="underline underline-offset-4 transition-colors hover:text-amber-400"
            href="https://1bridge.app/">1Bridge.app</a
          >
        </p>
      </div>
    </footer>
  </main>
</div>
