<script lang="ts">
  import { page } from '$app/state'
  import type {
    AuctionConfig,
    AuctionInfo,
    BidInfo,
    DepositTxInfo,
    StateInfo,
    UserInfo,
    WithdrawTxInfo,
    X402PaymentOutput
  } from '$declarations/ic_auction/ic_auction.did'
  import { icAuctionActor } from '$lib/canisters/icAuction'
  import CanisterModal from '$lib/components/CanisterModal.svelte'
  import CCAModal from '$lib/components/CCAModal.svelte'
  import Header from '$lib/components/Header.svelte'
  import ArrowRightUpLine from '$lib/icons/arrow-right-up-line.svelte'
  import InformationLine from '$lib/icons/information-line.svelte'
  import Settings4Line from '$lib/icons/settings-4-line.svelte'
  import { x402Settle } from '$lib/services/paying'
  import { authStore, EventLogin } from '$lib/stores/auth.svelte'
  import { showModal } from '$lib/stores/modal.svelte'
  import { toastRun, triggerToast } from '$lib/stores/toast.svelte'
  import { unwrapOption, unwrapResult } from '$lib/types/result'
  import Button from '$lib/ui/Button.svelte'
  import Spinner from '$lib/ui/Spinner.svelte'
  import { formatDatetime, parseUnits, pruneAddress } from '$lib/utils/helper'
  import { renderContent } from '$lib/utils/markdown'
  import {
    ICPToken,
    PANDAToken,
    TokenDisplay,
    type TokenInfo
  } from '$lib/utils/token'
  import {
    base64ToBytes,
    base64ToString,
    payingKit,
    type PaymentRequired,
    type PaymentRequirements
  } from '@ldclabs/1paying-kit'
  import { decode, encode, rfc8949EncodeOptions } from 'cborg'
  import { onDestroy, onMount, tick } from 'svelte'

  const canister = page.params['id'] || '4jxyd-pqaaa-aaaah-qdqtq-cai'
  const actor = icAuctionActor(canister)

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
  let x402Payment = $state<
    (X402PaymentOutput & { paymentRequired: PaymentRequired }) | null
  >(null)
  let payReq = $state<{
    payUrl: string
    txid: string
  } | null>(null)

  const tokenDisplay = $derived.by(() => new TokenDisplay(tokenInfo, 0n))
  const currencyDisplay = $derived.by(() => new TokenDisplay(currencyInfo, 0n))

  let myBids = $state<BidInfo[]>([])
  let myDeposits = $state<DepositTxInfo[]>([])
  let myWithdraws = $state<WithdrawTxInfo[]>([])

  // Inputs
  let floorPrice = $state(ICPToken.one)
  let bidAmount = $state('')
  let bidMaxPrice = $state('')

  let isDescriptionExpanded = $state(false)
  const descriptionText = $derived.by(() =>
    (stateInfo?.description || '').trim()
  )
  const isDescriptionLong = $derived.by(() => {
    if (!descriptionText) return false
    return (
      descriptionText.length > 280 || descriptionText.split('\n').length > 6
    )
  })

  $effect(() => {
    // Reset to collapsed when project/description changes
    descriptionText
    isDescriptionExpanded = false
  })
  let floorGroupedPrecision = $derived.by(() => {
    const val = Math.max(floorPrice.toString().length - 1, 1)
    return 10n ** BigInt(val)
  })
  let groupedPrecision = $state('')

  let withdrawCurrencyRecipient = $state('')
  let withdrawTokenRecipient = $state('')

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

  function getTokenUrl(address: string): string {
    if (!stateInfo) return ''
    if ('Icp' in stateInfo.chain) {
      return `https://www.icexplorer.io/token/details/${address}`
    } else if ('Sol' in stateInfo.chain) {
      switch (stateInfo.chain['Sol']) {
        case 0n:
          return `https://solscan.io/token/${address}?cluster=devnet`
        case 1n:
          return `https://solscan.io/token/${address}`
      }
    } else if ('Evm' in stateInfo.chain) {
      switch (stateInfo.chain['Evm']) {
        case 1n:
          return `https://etherscan.io/token/${address}`
        case 56n:
          return `https://bscscan.com/token/${address}`
        case 8453n:
          return `https://basescan.org/token/${address}`
        case 84532n:
          return `https://sepolia.basescan.org/token/${address}`
      }
    }
    return ''
  }

  function getAccountUrl(address: string): string {
    if (!stateInfo) return ''
    if ('Icp' in stateInfo.chain) {
      return `https://www.icexplorer.io/address/details/${address}`
    } else if ('Sol' in stateInfo.chain) {
      switch (stateInfo.chain['Sol']) {
        case 0n:
          return `https://solscan.io/address/${address}?cluster=devnet`
        case 1n:
          return `https://solscan.io/address/${address}`
      }
    } else if ('Evm' in stateInfo.chain) {
      switch (stateInfo.chain['Evm']) {
        case 1n:
          return `https://etherscan.io/address/${address}`
        case 56n:
          return `https://bscscan.com/address/${address}`
        case 8453n:
          return `https://basescan.org/address/${address}`
        case 84532n:
          return `https://sepolia.basescan.org/address/${address}`
      }
    }
    return ''
  }

  function getTxUrl(tx: string): string {
    if (!stateInfo) return ''
    if ('Icp' in stateInfo.chain) {
      return ''
    } else if ('Sol' in stateInfo.chain) {
      switch (stateInfo.chain['Sol']) {
        case 0n:
          return `https://solscan.io/tx/${tx}?cluster=devnet`
        case 1n:
          return `https://solscan.io/tx/${tx}`
      }
    } else if ('Evm' in stateInfo.chain) {
      switch (stateInfo.chain['Evm']) {
        case 1n:
          return `https://etherscan.io/tx/${tx}`
        case 56n:
          return `https://bscscan.com/tx/${tx}`
        case 8453n:
          return `https://basescan.org/tx/${tx}`
        case 84532n:
          return `https://sepolia.basescan.org/tx/${tx}`
      }
    }
    return ''
  }

  function getFinalizeKind(): [string, string] {
    if (!stateInfo) return ['—', '']
    if ('Transfer' in stateInfo.finalize_kind) {
      return [
        `Transfer to ${pruneAddress(stateInfo.funds_recipient, false)}`,
        getAccountUrl(stateInfo.funds_recipient)
      ]
    } else if ('CreatePool' in stateInfo.finalize_kind) {
      return [`Create pool on "${stateInfo.finalize_kind.CreatePool}"`, '']
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
      myDeposits = []
      myWithdraws = []
      return
    }
    const infoRes = await actor.my_info()
    myInfo = unwrapResult(infoRes, 'failed to fetch my info')

    const bidsRes = await actor.my_bids()
    myBids = unwrapResult(bidsRes, 'failed to fetch my bids')

    const depRes = await actor.my_deposits()
    myDeposits = unwrapResult(depRes, 'failed to fetch my deposits')

    const wdRes = await actor.my_withdraws()
    myWithdraws = unwrapResult(wdRes, 'failed to fetch my withdraws')

    await tick()
    if (myInfo.bound_addresses[0]) {
      withdrawCurrencyRecipient = myInfo.bound_addresses[0]
      withdrawTokenRecipient = myInfo.bound_addresses[0]
    }
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
      const priceAtomic = await actor.estimate_max_price(bidAmountUnits)
      bidMaxPriceLimit = priceAtomic
      bidMaxPrice = currencyDisplay.displayValue(
        priceAtomic + priceAtomic / 10n // add 10% buffer
      )
      const x402 = await actor.x402_payment(bidAmountUnits, false)
      const res = unwrapResult(
        x402,
        'failed to estimate x402 payment'
      ) as X402PaymentOutput & { paymentRequired: PaymentRequired }
      res.paymentRequired = decode(res.x402 as Uint8Array)
      x402Payment = res
      payReq = await payingKit.getPayUrl(res.paymentRequired)
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
      bidMaxPriceUnits > 0n &&
      (phase == 'prebid' || phase == 'bidding')
    )
  })

  function submitBid() {
    if (!stateInfo || !auctionCfg || !auctionInfo) {
      throw new Error('auction not ready')
    }
    if (!isBidable || !x402Payment || !payReq) return
    const payWindow = window.open(payReq.payUrl, '1paying-checkout')
    toastRun(async () => {
      if (!isBidable || !x402Payment || !payReq) return

      const paymentPayload = await payingKit.waitForPaymentPayload(payReq!.txid)

      if (payWindow && !payWindow.closed) {
        // Close the payment window if it's still open
        // We can not reuse the window for another payment due to browser security policies
        // Error: Blocked a frame with origin "https://1paying-coffee.zensh.workers.dev" from accessing a cross-origin frame.
        payWindow.close()
      }

      const x402Res = await x402Settle({
        paymentRequirements: x402Payment.paymentRequired
          .accepts[0] as PaymentRequirements,
        paymentPayload: JSON.parse(base64ToString(paymentPayload)),
        nonce: x402Payment.nonce
      })

      triggerToast({ type: 'success', message: 'Payment successful' })

      const depositRes = await actor.x402_deposit_currency({
        result: encode(x402Res.result, rfc8949EncodeOptions),
        signature: base64ToBytes(x402Res.signature),
        timestamp: x402Payment.timestamp
      })

      unwrapResult(depositRes, 'failed to deposit currency via 1Paying')
      triggerToast({ type: 'success', message: 'Deposit successful' })

      const res = await actor.submit_bid(bidAmountUnits, bidMaxPriceUnits)
      const bid = unwrapResult(res, 'submit bid failed')
      myBids.push(bid)
      triggerToast({ type: 'success', message: 'Bid has been submitted' })
      bidAmount = ''

      await refreshAuction()
      await refreshMine()
    }).finally(() => {
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

  async function withdrawCurrency() {
    const recipient = withdrawCurrencyRecipient.trim()
    if (!recipient) throw new Error('Please fill in recipient')
    const res = await actor.withdraw_currency({ recipient })
    unwrapResult(res, 'withdraw currency failed')
    triggerToast({ type: 'success', message: 'Currency withdrawal submitted' })
    withdrawCurrencyRecipient = ''
    await refreshMine()
  }

  async function withdrawToken() {
    const recipient = withdrawTokenRecipient.trim()
    if (!recipient) throw new Error('Please fill in recipient')
    const res = await actor.withdraw_token({ recipient })
    unwrapResult(res, 'withdraw token failed')
    triggerToast({ type: 'success', message: 'Token withdrawal submitted' })
    withdrawTokenRecipient = ''
    await refreshMine()
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

  let timer: any

  onMount(() => {
    return toastRun(async (_signal, abortingQue) => {
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
      // await refreshMine()
      authStore.addEventListener(EventLogin, refreshMine)
      abortingQue.push(() => {
        authStore.removeEventListener(EventLogin, refreshMine)
      })
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

  <Header description={'Continuous Clearing Auction'} />

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
              {#if descriptionText}
                <div class="space-y-2">
                  <div
                    class={`md-content w-full text-pretty wrap-break-word ${!isDescriptionExpanded && isDescriptionLong ? 'cca-desc-clamp' : ''}`}
                  >
                    {@html renderContent(descriptionText)}
                  </div>
                  {#if isDescriptionLong}
                    <button
                      class="inline-flex items-center gap-1 text-xs font-semibold tracking-wide text-violet-400 uppercase hover:text-violet-600"
                      onclick={() =>
                        (isDescriptionExpanded = !isDescriptionExpanded)}
                      type="button"
                    >
                      {isDescriptionExpanded ? 'Show less' : 'Show more'}
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
                class="border-border-subtle bg-surface rounded-lg border p-3"
              >
                <div
                  class="text-muted text-xs font-semibold tracking-wide uppercase"
                  >Phase</div
                >
                <div class="mt-1 text-lg font-bold">
                  {phase}
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
              {@const txUrl = getTxUrl(txid)}
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
                    href={getTokenUrl(currencyDisplay.token.address)}
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
                    href={getTokenUrl(tokenDisplay.token.address)}
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
                  <div class="text-muted">Total Supply</div>
                  <div class="font-semibold">
                    {tokenDisplay.displayValue(auctionCfg.total_supply)}
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
                        class="hover:border-foreground hover:text-foreground inline-flex items-center gap-1"
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
                  : '—'}
              </div>
            </div>
            <div class="border-border-subtle bg-surface rounded-lg border p-3">
              <div class="text-muted text-xs">Bids</div>
              <div class="mt-1 text-base font-bold">
                {auctionInfo ? auctionInfo.bids_count : '—'}
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
                <label
                  class="text-muted text-xs font-semibold tracking-wide uppercase"
                  for="bidAmount"
                >
                  Amount ({currencyInfo.symbol})
                </label>
                <input
                  id="bidAmount"
                  class="border-border-subtle bg-card mt-1 w-full rounded-lg border px-3 py-2 text-sm"
                  placeholder={`e.g. ${currencyDisplay.displayValue(auctionCfg.min_amount)} ${currencyInfo.symbol}`}
                  bind:value={bidAmount}
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
            <div class="text-muted text-sm"
              >View bids, claim/refunds, and review transaction records.</div
            >
          </div>
          <div class="flex items-center gap-2">
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

        <div class="mt-6 grid gap-6 lg:grid-cols-3">
          <div class="lg:col-span-2">
            <div
              class="text-muted mb-2 text-xs font-semibold tracking-wide uppercase"
              >My Bids</div
            >
            {#if !auctionInfo || myBids.length === 0}
              <div class="text-muted text-sm">No bids yet.</div>
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
                  {@const isClaimable =
                    !isClaimed &&
                    ((isOutbid && auctionInfo.is_graduated) ||
                      phase == 'ended')}
                  <div
                    class="border-border-subtle grid grid-cols-12 gap-2 border-t px-3 py-2 text-xs"
                  >
                    <div class="col-span-2 font-semibold">{b.id.toString()}</div
                    >
                    <div class="col-span-3">
                      {currencyDisplay.displayValue(b.amount)}
                      {currencyInfo.symbol}
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

          <div>
            <div
              class="text-muted mb-2 text-xs font-semibold tracking-wide uppercase"
              >Funds</div
            >
            <div class="space-y-4">
              <div
                class="border-border-subtle bg-surface rounded-xl border p-3"
              >
                <div class="flex items-center justify-between">
                  <span class="text-xs font-semibold">Withdraw Currency</span>
                  <span class="text-xs"
                    >{currencyDisplay.displayValue(myInfo.currency_amount)}
                    {currencyInfo.symbol}</span
                  >
                </div>
                <div class="mt-3 space-y-2">
                  <input
                    class="border-border-subtle bg-card w-full rounded-lg border px-3 py-2 text-xs"
                    placeholder="recipient"
                    bind:value={withdrawCurrencyRecipient}
                  />
                  <Button
                    class="border-border-subtle text-muted hover:border-foreground hover:text-foreground w-full rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
                    onclick={() =>
                      toastRun(withdrawCurrency, 'withdraw failed')}
                    disabled={myInfo.currency_amount == 0n ||
                      !withdrawCurrencyRecipient}
                  >
                    Withdraw
                  </Button>
                </div>
              </div>

              {#if myInfo.token_amount > 0n}
                <div
                  class="border-border-subtle bg-surface rounded-xl border p-3"
                >
                  <div class="flex items-center justify-between">
                    <span class="text-xs font-semibold">Withdraw Token</span>
                    <span class="text-xs"
                      >{tokenDisplay.displayValue(myInfo.token_amount)}
                      {tokenInfo.symbol}</span
                    >
                  </div>
                  <div class="mt-3 space-y-2">
                    <input
                      class="border-border-subtle bg-card w-full rounded-lg border px-3 py-2 text-xs"
                      placeholder="recipient"
                      bind:value={withdrawTokenRecipient}
                    />
                    <Button
                      class="border-border-subtle text-muted hover:border-foreground hover:text-foreground w-full rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
                      onclick={() => toastRun(withdrawToken, 'withdraw failed')}
                      disabled={myInfo.token_amount == 0n ||
                        !withdrawTokenRecipient}
                    >
                      Withdraw
                    </Button>
                  </div>
                </div>
              {/if}
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
              <div class="text-muted text-sm">No records yet.</div>
            {:else}
              <div
                class="border-border-subtle overflow-hidden rounded-xl border"
              >
                {#each myDeposits as d (d.txid)}
                  {@const txUrl = getTxUrl(d.txid)}
                  <div
                    class="border-border-subtle border-t px-3 py-2 text-xs first:border-t-0"
                  >
                    <div class="flex items-center justify-between">
                      <div class="font-semibold"
                        >{#if txUrl}
                          <a
                            class="hover:border-foreground hover:text-foreground inline-flex items-center gap-1"
                            href={txUrl}
                            target="_blank"
                            rel="noreferrer"
                          >
                            {pruneAddress(d.txid, true)}
                            <ArrowRightUpLine class="h-4 w-4" />
                          </a>
                        {:else}
                          {pruneAddress(d.txid, true)}
                        {/if}
                      </div>
                      <div class="text-muted">{formatDatetime(d.timestamp)}</div
                      >
                    </div>
                    <div class="text-muted mt-1">
                      {currencyDisplay.displayValue(d.amount)}
                      {currencyInfo.symbol} · sender: {pruneAddress(d.sender)}
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
              <div class="text-muted text-sm">No records yet.</div>
            {:else}
              <div
                class="border-border-subtle overflow-hidden rounded-xl border"
              >
                {#each myWithdraws as w (w.id)}
                  {@const txUrl = getTxUrl(w.txid)}
                  <div
                    class="border-border-subtle border-t px-3 py-2 text-xs first:border-t-0"
                  >
                    <div class="flex items-center justify-between">
                      <div class="font-semibold">#{w.id.toString()}</div>
                      <div class="text-muted">{formatDatetime(w.timestamp)}</div
                      >
                    </div>
                    <div class="text-muted mt-1">
                      {#if w.kind == 0}
                        <span
                          >{currencyDisplay.displayValue(w.amount)}
                          {currencyInfo.symbol}</span
                        >
                      {:else}
                        <span
                          >{tokenDisplay.displayValue(w.amount)}
                          {tokenInfo.symbol}</span
                        >
                      {/if}
                      <span
                        >{` · recipient:
                        ${pruneAddress(w.recipient)} · `}
                      </span>
                      {#if txUrl}
                        <a
                          class="hover:border-foreground hover:text-foreground inline-flex items-center gap-1"
                          href={txUrl}
                          target="_blank"
                          rel="noreferrer"
                        >
                          {pruneAddress(w.txid, true)}
                          <ArrowRightUpLine class="h-4 w-4" />
                        </a>
                      {:else}
                        {pruneAddress(w.txid, true)}
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
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
    -webkit-line-clamp: 4;
  }
</style>
