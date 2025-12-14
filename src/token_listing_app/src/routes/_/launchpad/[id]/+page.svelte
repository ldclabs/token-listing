<script lang="ts">
  import { page } from '$app/state'
  import { icAuctionActor } from '$lib/canisters/icAuction'
  import ArrowRightUpLine from '$lib/icons/arrow-right-up-line.svelte'
  import LogoutCircleRLine from '$lib/icons/logout-circle-r-line.svelte'
  import Wallet3Line from '$lib/icons/wallet-3-line.svelte'
  import { authStore } from '$lib/stores/auth.svelte'
  import { getTheme } from '$lib/stores/theme.svelte'
  import { toastRun, triggerToast } from '$lib/stores/toast.svelte'
  import { unwrapOption, unwrapResult } from '$lib/types/result'
  import Spinner from '$lib/ui/Spinner.svelte'
  import TextClipboardButton from '$lib/ui/TextClipboardButton.svelte'
  import { pruneAddress, pruneCanister } from '$lib/utils/helper'
  import Header from '$src/lib/components/Header.svelte'
  import { onDestroy, onMount } from 'svelte'

  import type {
    AuctionConfig,
    AuctionInfo,
    BidInfo,
    DepositTxInfo,
    StateInfo,
    WithdrawTxInfo
  } from '$declarations/ic_auction/ic_auction.did'

  const theme = $derived(getTheme())
  const actor = icAuctionActor(page.params['id'] || '')

  let loading = $state(true)
  let syncing = $state(false)

  let stateInfo = $state<StateInfo | null>(null)
  let auctionCfg = $state<AuctionConfig | null>(null)
  let auctionInfo = $state<AuctionInfo | null>(null)
  let grouped = $state<Array<[bigint, bigint]>>([])

  let myBids = $state<BidInfo[]>([])
  let myDeposits = $state<DepositTxInfo[]>([])
  let myWithdraws = $state<WithdrawTxInfo[]>([])

  // Inputs
  let bidAmount = $state('')
  let bidMaxPrice = $state('')
  let groupedPrecision = $state<'0.01' | '0.1' | '1' | '10'>('0.1')

  let depositSender = $state('')
  let depositTxid = $state('')
  let withdrawCurrencyRecipient = $state('')
  let withdrawTokenRecipient = $state('')

  const pow10 = (n: number) => 10n ** BigInt(Math.max(0, n))

  function formatUnits(amount: bigint, decimals: number, maxFrac = 4): string {
    const base = pow10(decimals)
    const sign = amount < 0n ? '-' : ''
    const value = amount < 0n ? -amount : amount
    const intPart = value / base
    const fracPart = value % base

    if (decimals === 0) return sign + intPart.toString()

    let frac = fracPart.toString().padStart(decimals, '0')
    if (maxFrac >= 0 && maxFrac < decimals) {
      frac = frac.slice(0, maxFrac)
    }
    frac = frac.replace(/0+$/, '')
    return frac.length
      ? `${sign}${intPart.toString()}.${frac}`
      : sign + intPart.toString()
  }

  function parseUnits(input: string, decimals: number): bigint {
    const s = input.trim().replace(/[,\s_']/g, '')
    if (!s) throw new Error('请输入数值')
    if (!/^\d*(?:\.\d*)?$/.test(s)) throw new Error('数值格式不正确')
    const [i = '0', f = ''] = s.split('.')
    const intPart = i ? BigInt(i) : 0n
    const frac = f.slice(0, decimals).padEnd(decimals, '0')
    const fracPart = frac ? BigInt(frac) : 0n
    return intPart * pow10(decimals) + fracPart
  }

  function formatMs(tsMs: bigint): string {
    const d = new Date(Number(tsMs))
    return d.toLocaleString()
  }

  function nowMs(): number {
    return Date.now()
  }

  function getPhase(cfg: AuctionConfig | null) {
    if (!cfg) return 'unconfigured' as const
    const n = BigInt(nowMs())
    if (n < cfg.start_time) return 'upcoming' as const
    if (n > cfg.end_time) return 'ended' as const
    return 'running' as const
  }

  function progress(cfg: AuctionConfig | null): number {
    if (!cfg) return 0
    const n = BigInt(nowMs())
    const start = cfg.start_time
    const end = cfg.end_time
    if (end <= start) return 0
    if (n <= start) return 0
    if (n >= end) return 1
    const done = Number(n - start)
    const total = Number(end - start)
    return Math.max(0, Math.min(1, done / total))
  }

  function currencyDecimals() {
    return stateInfo?.currency_decimals ?? 0
  }

  function tokenDecimals() {
    return stateInfo?.token_decimals ?? 0
  }

  function currencySymbol() {
    return stateInfo?.currency_symbol ?? 'CUR'
  }

  function tokenSymbol() {
    return stateInfo?.token_symbol ?? 'TKN'
  }

  function priceUnitsPerToken(priceAtomic: bigint): string {
    return `${formatUnits(priceAtomic, currencyDecimals(), 6)} ${currencySymbol()}/${tokenSymbol()}`
  }

  async function refreshAll() {
    syncing = true
    try {
      const sres = await actor.info()
      const s = unwrapResult(sres, 'failed to fetch auction state')
      stateInfo = s
      auctionCfg = unwrapOption(s.auction_config)

      const ai = unwrapOption(await actor.auction_info())
      auctionInfo = ai

      await refreshGrouped()
      await refreshMine()
    } finally {
      syncing = false
      loading = false
    }
  }

  async function refreshGrouped() {
    const dec = currencyDecimals()
    const precisionAtomic =
      groupedPrecision === '0.01'
        ? pow10(dec - 2)
        : groupedPrecision === '0.1'
          ? pow10(dec - 1)
          : groupedPrecision === '1'
            ? pow10(dec)
            : pow10(dec + 1)

    grouped = await actor.get_grouped_bids([precisionAtomic])
  }

  async function refreshMine() {
    if (!authStore.identity.isAuthenticated) {
      myBids = []
      myDeposits = []
      myWithdraws = []
      return
    }
    const bidsRes = await actor.my_bids()
    myBids = unwrapResult(bidsRes, 'failed to fetch my bids')

    const depRes = await actor.my_deposits()
    myDeposits = unwrapResult(depRes, 'failed to fetch my deposits')

    const wdRes = await actor.my_withdraws()
    myWithdraws = unwrapResult(wdRes, 'failed to fetch my withdraws')
  }

  async function estimateMaxPrice() {
    if (!stateInfo) throw new Error('state not ready')
    const amountAtomic = parseUnits(bidAmount, currencyDecimals())
    const priceAtomic = await actor.estimate_max_price(amountAtomic)
    bidMaxPrice = formatUnits(priceAtomic, currencyDecimals(), 6)
  }

  async function submitBid() {
    if (!stateInfo || !auctionCfg) throw new Error('auction not ready')
    if (!authStore.identity.isAuthenticated) {
      triggerToast({ type: 'info', message: '请先连接身份（Sign in）' })
      return
    }
    if (getPhase(auctionCfg) !== 'running') {
      throw new Error('Auction 不在进行中')
    }
    const amountAtomic = parseUnits(bidAmount, currencyDecimals())
    const maxPriceAtomic = parseUnits(bidMaxPrice, currencyDecimals())

    const res = await actor.submit_bid(amountAtomic, maxPriceAtomic)
    unwrapResult(res, 'submit bid failed')
    triggerToast({ type: 'success', message: '出价已提交' })
    bidAmount = ''
    // keep max price
    await refreshAll()
  }

  async function claimOne(id: bigint) {
    const res = await actor.claim(id)
    unwrapResult(res, 'claim failed')
    triggerToast({ type: 'success', message: '已领取/退款完成' })
    await refreshAll()
  }

  async function claimAll() {
    const res = await actor.claim_all()
    unwrapResult(res, 'claim all failed')
    triggerToast({ type: 'success', message: '已批量领取/退款完成' })
    await refreshAll()
  }

  async function depositCurrency() {
    if (!authStore.identity.isAuthenticated) {
      triggerToast({ type: 'info', message: '请先连接身份（Sign in）' })
      return
    }
    if (!depositSender.trim() || !depositTxid.trim()) {
      throw new Error('请填写 sender 与 txid')
    }
    const balRes = await actor.deposit_currency({
      sender: depositSender.trim(),
      txid: depositTxid.trim()
    })
    const balance = unwrapResult(balRes, 'deposit failed')
    triggerToast({
      type: 'success',
      message: `Deposit 成功，余额：${formatUnits(balance, currencyDecimals(), 6)} ${currencySymbol()}`
    })
    depositSender = ''
    depositTxid = ''
    await refreshAll()
  }

  async function withdrawCurrency() {
    if (!authStore.identity.isAuthenticated) {
      triggerToast({ type: 'info', message: '请先连接身份（Sign in）' })
      return
    }
    const recipient = withdrawCurrencyRecipient.trim()
    if (!recipient) throw new Error('请填写 recipient')
    const res = await actor.withdraw_currency({ recipient })
    unwrapResult(res, 'withdraw currency failed')
    triggerToast({ type: 'success', message: '已提交 currency 提现' })
    withdrawCurrencyRecipient = ''
    await refreshAll()
  }

  async function withdrawToken() {
    if (!authStore.identity.isAuthenticated) {
      triggerToast({ type: 'info', message: '请先连接身份（Sign in）' })
      return
    }
    const recipient = withdrawTokenRecipient.trim()
    if (!recipient) throw new Error('请填写 recipient')
    const res = await actor.withdraw_token({ recipient })
    unwrapResult(res, 'withdraw token failed')
    triggerToast({ type: 'success', message: '已提交 token 提现' })
    withdrawTokenRecipient = ''
    await refreshAll()
  }

  let timer: any

  onMount(() => {
    toastRun(async () => {
      await refreshAll()
    }, '初始化失败')

    timer = setInterval(() => {
      toastRun(async () => {
        await refreshAll()
      })
    }, 2500)
  })

  onDestroy(() => {
    if (timer) clearInterval(timer)
  })

  const signIn = () =>
    toastRun(async () => {
      await authStore.signIn()
      await refreshAll()
    }, '登录失败')

  const logout = () =>
    toastRun(async () => {
      await authStore.logout()
      await refreshAll()
    }, '登出失败')
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

  <Header>
    {#if authStore.identity.isAuthenticated}
      <div
        class="border-border-subtle bg-surface flex items-center gap-2 rounded-full border px-3 py-1 text-xs"
      >
        <Wallet3Line class="h-4 w-4" />
        <span class="font-semibold"
          >{pruneCanister(authStore.identity.getPrincipal().toText())}</span
        >
        <TextClipboardButton
          value={authStore.identity.getPrincipal().toText()}
          class="text-muted hover:text-foreground"
          ariaLabel="Copy principal"
        />
      </div>
      <button
        class="border-border-subtle text-muted hover:border-foreground hover:text-foreground inline-flex items-center gap-2 rounded-full border px-4 py-2 font-semibold tracking-wide uppercase transition-all"
        onclick={logout}
      >
        <LogoutCircleRLine class="h-4 w-4" />
        Logout
      </button>
    {:else}
      <button
        class="bg-foreground text-background inline-flex items-center gap-2 rounded-full px-4 py-2 font-semibold tracking-wide uppercase transition-all hover:opacity-90"
        onclick={signIn}
      >
        <Wallet3Line class="h-4 w-4" />
        Sign in
      </button>
    {/if}
  </Header>

  <main
    class="relative z-10 mx-auto w-full max-w-6xl space-y-6 px-4 py-6 md:px-8 md:py-10"
  >
    {#if loading}
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
            <div
              class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between"
            >
              <div class="space-y-1">
                <div
                  class="text-muted text-xs font-semibold tracking-wide uppercase"
                  >Project</div
                >
                <div class="text-xl font-bold">
                  {stateInfo?.name || 'Auction'}
                  {#if stateInfo?.token_symbol}
                    <span class="text-muted font-semibold"
                      >· {stateInfo.token_symbol}</span
                    >
                  {/if}
                </div>
                <div class="text-muted text-sm"
                  >{stateInfo?.description || '—'}</div
                >
              </div>
              <div class="flex items-center gap-2">
                {#if stateInfo?.url}
                  <a
                    class="border-border-subtle text-muted hover:border-foreground hover:text-foreground inline-flex items-center gap-2 rounded-full border px-3 py-1 text-xs font-semibold tracking-wide uppercase"
                    href={stateInfo.url}
                    target="_blank"
                    rel="noreferrer"
                  >
                    Website
                    <ArrowRightUpLine class="h-4 w-4" />
                  </a>
                {/if}
              </div>
            </div>

            <div class="grid gap-3 sm:grid-cols-3">
              <div
                class="border-border-subtle bg-surface rounded-lg border p-3"
              >
                <div
                  class="text-muted text-xs font-semibold tracking-wide uppercase"
                  >Phase</div
                >
                <div class="mt-1 text-lg font-bold">
                  {#if auctionCfg}
                    {getPhase(auctionCfg)}
                  {:else}
                    unconfigured
                  {/if}
                </div>
              </div>
              <div
                class="border-border-subtle bg-surface rounded-lg border p-3"
              >
                <div
                  class="text-muted text-xs font-semibold tracking-wide uppercase"
                  >Clearing Price</div
                >
                <div class="mt-1 text-lg font-bold">
                  {#if auctionInfo}
                    {priceUnitsPerToken(auctionInfo.clearing_price)}
                  {:else}
                    —
                  {/if}
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
                  {auctionInfo ? auctionInfo.bidders_count.toString() : '—'}
                </div>
              </div>
            </div>

            {#if auctionCfg}
              <div class="space-y-2">
                <div class="flex items-center justify-between text-xs">
                  <div class="text-muted"
                    >{formatMs(auctionCfg.start_time)} → {formatMs(
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
          </div>
        </div>

        <div class="glass-border rounded-xl p-4 md:p-6">
          <div class="space-y-4">
            <div
              class="text-muted text-xs font-semibold tracking-wide uppercase"
              >Targets</div
            >
            {#if auctionCfg}
              <div class="space-y-2">
                <div class="flex items-center justify-between text-sm">
                  <div class="text-muted">Graduation Threshold</div>
                  <div class="font-semibold">
                    {formatUnits(
                      auctionCfg.required_currency_raised,
                      currencyDecimals(),
                      6
                    )}
                    {currencySymbol()}
                  </div>
                </div>
                <div class="flex items-center justify-between text-sm">
                  <div class="text-muted">Total Supply</div>
                  <div class="font-semibold">
                    {formatUnits(auctionCfg.total_supply, tokenDecimals(), 6)}
                    {tokenSymbol()}
                  </div>
                </div>
                <div class="flex items-center justify-between text-sm">
                  <div class="text-muted">Min/Max Bid</div>
                  <div class="font-semibold">
                    {formatUnits(
                      auctionCfg.min_amount,
                      currencyDecimals(),
                      6
                    )}–{formatUnits(
                      auctionCfg.max_amount,
                      currencyDecimals(),
                      6
                    )}
                    {currencySymbol()}
                  </div>
                </div>
                <div class="flex items-center justify-between text-sm">
                  <div class="text-muted">Min Bid Duration</div>
                  <div class="font-semibold"
                    >{(Number(auctionCfg.min_bid_duration) / 1000).toFixed(
                      0
                    )}s</div
                  >
                </div>
              </div>
            {:else}
              <div class="text-muted text-sm"
                >Auction 尚未配置（`auction_config` 为空）</div
              >
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
                  ? `${formatUnits(auctionInfo.cumulative_demand_raised, currencyDecimals(), 6)} ${currencySymbol()}`
                  : '—'}
              </div>
            </div>
            <div class="border-border-subtle bg-surface rounded-lg border p-3">
              <div class="text-muted text-xs">Supply Released</div>
              <div class="mt-1 text-base font-bold">
                {auctionInfo
                  ? `${formatUnits(auctionInfo.cumulative_supply_released, tokenDecimals(), 6)} ${tokenSymbol()}`
                  : '—'}
              </div>
            </div>
            <div class="border-border-subtle bg-surface rounded-lg border p-3">
              <div class="text-muted text-xs">Total Tokens Filled</div>
              <div class="mt-1 text-base font-bold">
                {auctionInfo
                  ? `${formatUnits(auctionInfo.total_tokens_filled, tokenDecimals(), 6)} ${tokenSymbol()}`
                  : '—'}
              </div>
            </div>
            <div class="border-border-subtle bg-surface rounded-lg border p-3">
              <div class="text-muted text-xs">Graduated</div>
              <div class="mt-1 text-base font-bold">
                {auctionInfo ? (auctionInfo.is_graduated ? 'Yes' : 'No') : '—'}
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
                  class="border-border-subtle bg-card rounded-lg border px-2 py-1 text-xs"
                  bind:value={groupedPrecision}
                  oninput={() => toastRun(refreshGrouped)}
                >
                  <option value="0.01">0.01</option>
                  <option value="0.1">0.1</option>
                  <option value="1">1</option>
                  <option value="10">10</option>
                </select>
                <span class="text-muted text-xs"
                  >{currencySymbol()}/{tokenSymbol()}</span
                >
              </div>
            </div>

            {#if grouped.length === 0}
              <div class="text-muted mt-3 text-sm"
                >暂无可视化数据（没有活跃出价或精度过细）</div
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
                      ≤ {formatUnits(p, currencyDecimals(), 6)}
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
                      {formatUnits(a, currencyDecimals(), 6)}
                      {currencySymbol()}
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
            <div class="text-muted text-sm">
              `max_price` 以 “每 1 个 {tokenSymbol()} 需要多少 {currencySymbol()}”
              计价。
            </div>
          </div>

          {#if !auctionCfg}
            <div class="text-muted mt-4 text-sm"
              >Auction 尚未配置，暂不可出价。</div
            >
          {:else}
            <div class="mt-4 grid gap-3">
              <div>
                <label
                  class="text-muted text-xs font-semibold tracking-wide uppercase"
                  for="bidAmount"
                >
                  Amount ({currencySymbol()})
                </label>
                <input
                  id="bidAmount"
                  class="border-border-subtle bg-card mt-1 w-full rounded-lg border px-3 py-2 text-sm"
                  placeholder={`e.g. 100 ${currencySymbol()}`}
                  bind:value={bidAmount}
                  inputmode="decimal"
                />
              </div>

              <div>
                <label
                  class="text-muted text-xs font-semibold tracking-wide uppercase"
                  for="bidMaxPrice"
                >
                  Max Price ({currencySymbol()}/{tokenSymbol()})
                </label>
                <input
                  id="bidMaxPrice"
                  class="border-border-subtle bg-card mt-1 w-full rounded-lg border px-3 py-2 text-sm"
                  placeholder={`e.g. 2.5 ${currencySymbol()}/${tokenSymbol()}`}
                  bind:value={bidMaxPrice}
                  inputmode="decimal"
                />
              </div>

              <div class="flex flex-wrap items-center gap-2">
                <button
                  class="border-border-subtle text-muted hover:border-foreground hover:text-foreground inline-flex items-center gap-2 rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
                  onclick={() => toastRun(estimateMaxPrice, 'estimate failed')}
                  disabled={!bidAmount}
                >
                  Estimate
                </button>

                <button
                  class="bg-foreground text-background inline-flex flex-1 items-center justify-center gap-2 rounded-full px-4 py-2 text-xs font-semibold tracking-wide uppercase hover:opacity-90"
                  onclick={() => toastRun(submitBid, 'submit failed')}
                  disabled={!bidAmount || !bidMaxPrice}
                >
                  Submit Bid
                </button>
              </div>

              <div
                class="border-border-subtle bg-surface rounded-lg border p-3 text-xs"
              >
                <div class="text-muted">Tip</div>
                <div class="mt-1">
                  出价是“按剩余时间均摊”的流式出价：越早参与越能更长时间暴露资金；如果清算价超过你的
                  `max_price`，将被 outbid 并可领取未花费部分退款。
                </div>
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
            <div class="text-muted text-sm"
              >查看出价、领取/退款、以及资金进出记录。</div
            >
          </div>
          <div class="flex items-center gap-2">
            <button
              class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
              onclick={() => toastRun(claimAll, 'claim all failed')}
              disabled={!authStore.identity.isAuthenticated ||
                myBids.length === 0}
            >
              Claim All
            </button>
          </div>
        </div>

        {#if !authStore.identity.isAuthenticated}
          <div class="text-muted mt-4 text-sm"
            >请先 Sign in 以查看你的出价与记录。</div
          >
        {:else}
          <div class="mt-6 grid gap-6 lg:grid-cols-3">
            <div class="lg:col-span-2">
              <div
                class="text-muted mb-2 text-xs font-semibold tracking-wide uppercase"
                >My Bids</div
              >
              {#if myBids.length === 0}
                <div class="text-muted text-sm">暂无出价</div>
              {:else}
                <div
                  class="border-border-subtle overflow-hidden rounded-xl border"
                >
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
                    <div
                      class="border-border-subtle grid grid-cols-12 gap-2 border-t px-3 py-2 text-xs"
                    >
                      <div class="col-span-2 font-semibold"
                        >{b.id.toString()}</div
                      >
                      <div class="col-span-3">
                        {formatUnits(b.amount, currencyDecimals(), 6)}
                        {currencySymbol()}
                      </div>
                      <div class="col-span-3"
                        >{priceUnitsPerToken(b.max_price)}</div
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
                        <button
                          class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-2 py-1 text-xs font-semibold"
                          onclick={() =>
                            toastRun(() => claimOne(b.id), 'claim failed')}
                          disabled={isClaimed}
                        >
                          Claim
                        </button>
                      </div>
                    </div>
                    {#if b.tokens_filled > 0n || b.refund > 0n}
                      <div
                        class="border-border-subtle bg-surface grid grid-cols-12 gap-2 border-t px-3 py-2 text-xs"
                      >
                        <div class="text-muted col-span-12">
                          Filled: <span class="text-foreground font-semibold"
                            >{formatUnits(b.tokens_filled, tokenDecimals(), 6)}
                            {tokenSymbol()}</span
                          >
                          · Refund:
                          <span class="text-foreground font-semibold"
                            >{formatUnits(b.refund, currencyDecimals(), 6)}
                            {currencySymbol()}</span
                          >
                        </div>
                      </div>
                    {/if}
                  {/each}
                </div>
              {/if}
            </div>

            <div>
              <div
                class="text-muted mb-2 text-xs font-semibold tracking-wide uppercase"
                >Funds</div
              >
              <div class="space-y-4">
                <div
                  class="border-border-subtle bg-surface rounded-xl border p-3"
                >
                  <div class="text-xs font-semibold">Deposit Currency</div>
                  <div class="text-muted mt-1 text-xs">
                    使用后端 `deposit_currency(sender, txid)`
                    录入外部链转账（或桥接）记录。
                  </div>
                  <div class="mt-3 space-y-2">
                    <input
                      class="border-border-subtle bg-card w-full rounded-lg border px-3 py-2 text-xs"
                      placeholder="sender"
                      bind:value={depositSender}
                    />
                    <input
                      class="border-border-subtle bg-card w-full rounded-lg border px-3 py-2 text-xs"
                      placeholder="txid"
                      bind:value={depositTxid}
                    />
                    <button
                      class="bg-foreground text-background w-full rounded-full px-3 py-2 text-xs font-semibold tracking-wide uppercase hover:opacity-90"
                      onclick={() =>
                        toastRun(depositCurrency, 'deposit failed')}
                      disabled={!depositSender || !depositTxid}
                    >
                      Deposit
                    </button>
                  </div>
                </div>

                <div
                  class="border-border-subtle bg-surface rounded-xl border p-3"
                >
                  <div class="text-xs font-semibold">Withdraw Currency</div>
                  <div class="mt-3 space-y-2">
                    <input
                      class="border-border-subtle bg-card w-full rounded-lg border px-3 py-2 text-xs"
                      placeholder="recipient"
                      bind:value={withdrawCurrencyRecipient}
                    />
                    <button
                      class="border-border-subtle text-muted hover:border-foreground hover:text-foreground w-full rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
                      onclick={() =>
                        toastRun(withdrawCurrency, 'withdraw failed')}
                      disabled={!withdrawCurrencyRecipient}
                    >
                      Withdraw
                    </button>
                  </div>
                </div>

                <div
                  class="border-border-subtle bg-surface rounded-xl border p-3"
                >
                  <div class="text-xs font-semibold">Withdraw Token</div>
                  <div class="mt-3 space-y-2">
                    <input
                      class="border-border-subtle bg-card w-full rounded-lg border px-3 py-2 text-xs"
                      placeholder="recipient"
                      bind:value={withdrawTokenRecipient}
                    />
                    <button
                      class="border-border-subtle text-muted hover:border-foreground hover:text-foreground w-full rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
                      onclick={() => toastRun(withdrawToken, 'withdraw failed')}
                      disabled={!withdrawTokenRecipient}
                    >
                      Withdraw
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="mt-8 grid gap-6 lg:grid-cols-2">
            <div>
              <div
                class="text-muted mb-2 text-xs font-semibold tracking-wide uppercase"
                >Deposits</div
              >
              {#if myDeposits.length === 0}
                <div class="text-muted text-sm">暂无记录</div>
              {:else}
                <div
                  class="border-border-subtle overflow-hidden rounded-xl border"
                >
                  {#each myDeposits as d (d.txid)}
                    <div
                      class="border-border-subtle border-t px-3 py-2 text-xs first:border-t-0"
                    >
                      <div class="flex items-center justify-between">
                        <div class="font-semibold"
                          >{pruneAddress(d.txid, true)}</div
                        >
                        <div class="text-muted"
                          >{new Date(Number(d.timestamp)).toLocaleString()}</div
                        >
                      </div>
                      <div class="text-muted mt-1">
                        {formatUnits(d.amount, currencyDecimals(), 6)}
                        {currencySymbol()} · sender: {pruneAddress(d.sender)}
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

            <div>
              <div
                class="text-muted mb-2 text-xs font-semibold tracking-wide uppercase"
                >Withdraws</div
              >
              {#if myWithdraws.length === 0}
                <div class="text-muted text-sm">暂无记录</div>
              {:else}
                <div
                  class="border-border-subtle overflow-hidden rounded-xl border"
                >
                  {#each myWithdraws as w (w.id)}
                    <div
                      class="border-border-subtle border-t px-3 py-2 text-xs first:border-t-0"
                    >
                      <div class="flex items-center justify-between">
                        <div class="font-semibold">#{w.id.toString()}</div>
                        <div class="text-muted"
                          >{new Date(Number(w.timestamp)).toLocaleString()}</div
                        >
                      </div>
                      <div class="text-muted mt-1">
                        {formatUnits(w.amount, currencyDecimals(), 6)} · recipient:
                        {pruneAddress(w.recipient)} · tx: {pruneAddress(
                          w.txid,
                          true
                        )}
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </section>
    {/if}
  </main>
</div>
