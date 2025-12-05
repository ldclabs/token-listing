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
      tag: 'Metadata'
    },
    {
      title: 'Cross-Chain Minting',
      desc: 'Issue tokens on EVM, Solana, or ICP from a single dashboard. Manage supply and permissions across all chains.',
      tag: 'Mint'
    },
    {
      title: 'Fair Continuous Clearing Auctions',
      desc: 'CCA spreads bids over time automatically. Early supporters get fair prices; bots and snipers get nothing.',
      tag: 'Launch'
    },
    {
      title: 'On-Chain Governance',
      desc: 'Community-driven decisions with transparent, executable on-chain proposals. No hidden multisigs, no backroom deals.',
      tag: 'DAO'
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
    { name: 'ICP', note: 'Orchestration layer & permanent storage' },
    { name: 'EVM', note: 'Ethereum & EVM-compatible issuance and bridges' },
    { name: 'Solana', note: 'High-speed SPL token support' },
    { name: 'More', note: 'Expanding to L1s and appchains' }
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

<div class="relative flex min-h-screen flex-col">
  <!-- Header -->
  <header
    class="border-border-subtle sticky top-0 z-50 border-b shadow-sm backdrop-blur"
  >
    <div
      class="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-4 px-4 py-2 md:px-8"
    >
      <div class="flex items-center gap-3">
        <img
          alt="TokenList.ing Logo"
          src="/_assets/logo.webp"
          class="h-12 w-12 rounded-sm bg-white object-contain p-0.5"
        />

        <div>
          <p class="text-muted text-lg tracking-[0.18em]">TokenList.ing</p>
          <p class="text-foreground hidden text-base font-medium md:flex">
            Token infrastructure for the multi-chain era
          </p>
        </div>
      </div>

      <nav
        class="text-muted hidden items-center gap-6 text-sm font-medium md:flex"
      >
        {#each navLinks as link}
          <a class="hover:text-foreground transition" href={link.href}>
            {link.label}
          </a>
        {/each}
        <a
          class="border-border-subtle hover:border-foreground hover:bg-accent hover:text-accent-foreground rounded-full border px-4 py-2 text-sm font-semibold transition"
          href="#cta"
        >
          Get early access
        </a>
      </nav>

      <button
        class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase transition"
        onclick={toggleTheme}
        aria-pressed={theme === 'light'}
        aria-label="Toggle theme"
      >
        {theme === 'dark' ? '☀️' : '🌙'}
      </button>
    </div>
  </header>

  <main class="mx-auto mt-10 max-w-6xl space-y-10 px-4 md:mt-20 md:px-8">
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
            class="text-4xl leading-[1.1] font-semibold sm:text-5xl lg:text-6xl"
          >
            Launch tokens fairly.<br class="hidden sm:block" />
            Store metadata forever.
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
            class="bg-accent text-accent-foreground rounded-full px-6 py-3 text-sm font-semibold tracking-wide uppercase transition hover:-translate-y-0.5 hover:shadow-lg"
            href="#launchpad"
          >
            How CCA works
          </a>
          <a
            class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-6 py-3 text-sm font-semibold transition"
            href="#features"
          >
            All features
          </a>
        </div>
      </div>

      <!-- Live Auction Preview -->
      <div class="relative">
        <div
          class="glass-border from-surface-hover relative overflow-hidden rounded-xl bg-linear-to-b via-transparent to-transparent p-4 md:p-8"
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
              <p class="font-mono text-3xl font-medium">0.0847 USDC</p>
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

            <div class="bg-surface rounded-xl p-4">
              <p class="text-muted-foreground text-xs">
                Price adjusts continuously as bids come in. Your final cost is
                the clearing price at auction end—same for everyone.
              </p>
            </div>
          </div>

          <div
            class="bg-surface-hover absolute -right-10 -bottom-10 h-40 w-40 rounded-full blur-2xl"
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
          <h2 class="mt-2 text-3xl font-semibold sm:text-4xl">
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

      <div class="grid gap-5 md:grid-cols-2">
        {#each pillars as item}
          <div
            class="glass-border group relative overflow-hidden rounded-xl p-4 transition hover:-translate-y-1 md:p-8"
          >
            <div
              class="bg-surface-hover group-hover:bg-border-subtle absolute top-0 right-0 h-24 w-24 translate-x-8 -translate-y-4 rounded-full blur-2xl transition"
            ></div>
            <div
              class="border-border-subtle bg-surface text-muted mb-4 inline-flex items-center rounded-full border px-3 py-1 text-[10px] font-bold tracking-widest uppercase"
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
        class="border-border-subtle bg-surface space-y-10 rounded-xl border p-4 md:p-8"
      >
        <div
          class="flex flex-col gap-4 md:flex-row md:items-end md:justify-between"
        >
          <div>
            <p class="text-muted text-sm tracking-widest uppercase">
              Continuous Clearing Auction
            </p>
            <h2 class="mt-2 text-3xl font-semibold sm:text-4xl">
              A better way to launch
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

        <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
          {#each steps as step, i}
            <div class="glass-border flex flex-col gap-3 rounded-xl p-4">
              <div
                class="bg-border-subtle flex h-8 w-8 items-center justify-center rounded-full text-xs font-bold"
              >
                {i + 1}
              </div>
              <h3 class="font-semibold">{step.title}</h3>
              <p class="text-muted text-xs leading-relaxed">{step.body}</p>
            </div>
          {/each}
        </div>

        <div class="grid gap-6 md:grid-cols-2">
          <div class="glass-border rounded-xl p-4">
            <p class="text-muted text-sm tracking-widest uppercase">
              For projects
            </p>
            <ul class="text-muted mt-4 space-y-3 text-sm">
              <li>
                → <strong class="text-foreground">Predictable outcomes</strong> —
                know your minimum raise before you start
              </li>
              <li>
                → <strong class="text-foreground">Instant liquidity</strong> — auction
                proceeds seed your trading pool automatically
              </li>
              <li>
                → <strong class="text-foreground">Credible distribution</strong> —
                prove to your community that insiders didn't front-run
              </li>
            </ul>
          </div>
          <div class="glass-border rounded-xl p-4">
            <p class="text-muted text-sm tracking-widest uppercase">
              For participants
            </p>
            <ul class="text-muted mt-4 space-y-3 text-sm">
              <li>
                → <strong class="text-foreground">No timing pressure</strong> — bid
                early, bid late, same fair treatment
              </li>
              <li>
                → <strong class="text-foreground">Set your max price</strong> — if
                clearing exceeds your limit, you get refunded
              </li>
              <li>
                → <strong class="text-foreground">Full refund guarantee</strong> —
                failed auctions return 100% of funds
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
          <h2 class="mt-2 text-3xl font-semibold sm:text-4xl">
            One platform, every chain
          </h2>
        </div>
        <p class="text-muted text-sm">
          Powered by ICP's Chain-Key & HTTPs-Outcall capabilities
        </p>
      </div>

      <div class="grid gap-4 md:grid-cols-4">
        {#each chains as chain}
          <div class="glass-border rounded-xl p-4">
            <div class="flex items-center justify-between">
              <span class="font-semibold">{chain.name}</span>
              <span class="text-muted-foreground">→</span>
            </div>
            <p class="text-muted mt-2 text-xs">{chain.note}</p>
          </div>
        {/each}
      </div>

      <div class="glass-border rounded-xl p-4 md:p-8">
        <div class="grid gap-6 md:grid-cols-2">
          <div>
            <p class="text-muted text-sm tracking-widest uppercase">
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
            <p class="text-muted text-sm tracking-widest uppercase">
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
          <h2 class="text-2xl font-semibold sm:text-3xl">
            Frequently asked questions
          </h2>
          <a
            class="text-muted hover:text-foreground text-sm underline underline-offset-4"
            href="mailto:hello@tokenlist.ing"
          >
            More questions?
          </a>
        </div>
        <div class="space-y-4">
          {#each faqs as item}
            <div class="glass-border rounded-xl p-4">
              <p class="text-foreground font-semibold">{item.q}</p>
              <p class="text-muted mt-2 text-sm leading-relaxed">{item.a}</p>
            </div>
          {/each}
        </div>
      </div>
    </section>

    <!-- CTA Section -->
    <section
      id="cta"
      class="border-border-subtle from-surface-hover via-surface to-surface-hover rounded-xl border bg-linear-to-r p-4 pt-10 text-center md:p-8 md:pt-20"
    >
      <p class="text-muted text-sm tracking-widest uppercase"> Early access </p>
      <h2 class="mt-3 text-3xl font-semibold sm:text-4xl">
        Ready to launch fairly?
      </h2>
      <p class="text-muted mx-auto mt-4 max-w-2xl text-sm">
        We're onboarding the first wave of projects now. Tell us about your
        token, your community, and your timeline—we'll help you configure the
        perfect launch.
      </p>
      <div class="mt-8 flex flex-wrap justify-center gap-4">
        <a
          class="bg-accent text-accent-foreground rounded-full px-6 py-3 text-sm font-semibold tracking-wide uppercase transition hover:-translate-y-0.5"
          href="https://x.com/ICPandaDAO"
        >
          Get in touch
        </a>
        <a
          class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-6 py-3 text-sm font-semibold transition"
          href="https://github.com/ldclabs/token-listing/blob/main/docs/cca.md"
        >
          Read the docs
        </a>
      </div>
    </section>

    <!-- Footer -->
    <footer class="border-border-subtle border-t py-10 md:py-20">
      <div class="flex flex-col items-center">
        <p class="flex flex-row items-center gap-1">
          <span class="text-sm">© {new Date().getFullYear()}</span>
          <a class="" href="https://panda.fans" target="_blank"
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
            class="underline underline-offset-4"
            href="https://anda.ai"
            target="_blank">Anda.AI</a
          >
          <span class="mx-1">|</span>
          <a class="underline underline-offset-4" href="https://dmsg.net"
            >dMsg.net</a
          >
          <span class="mx-1">|</span>
          <a class="underline underline-offset-4" href="https://1bridge.app/"
            >1Bridge.app</a
          >
        </p>
      </div>
    </footer>
  </main>
</div>
