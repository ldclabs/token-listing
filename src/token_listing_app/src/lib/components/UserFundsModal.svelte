<script lang="ts">
  import type {
    DepositTxInfo,
    StateInfo,
    UserInfo,
    WithdrawTxInfo,
    X402PaymentOutput
  } from '$declarations/ic_auction/ic_auction.did'
  import { type AuctionService } from '$lib/canisters/icAuction'
  import ArrowRightUpLine from '$lib/icons/arrow-right-up-line.svelte'
  import { x402Settle, x402Verify } from '$lib/services/paying'
  import { authStore } from '$lib/stores/auth.svelte'
  import { toastRun, triggerToast } from '$lib/stores/toast.svelte'
  import { unwrapResult } from '$lib/types/result'
  import Button from '$lib/ui/Button.svelte'
  import TextClipboardButton from '$lib/ui/TextClipboardButton.svelte'
  import { getTxUrl } from '$lib/utils/chain'
  import {
    formatDatetime,
    parseUnits,
    pruneAddress,
    sleep
  } from '$lib/utils/helper'
  import { TokenDisplay, type TokenInfo } from '$lib/utils/token'
  import {
    base64ToBytes,
    base64ToString,
    payingKit,
    type PaymentRequired,
    type PaymentRequirements
  } from '@ldclabs/1paying-kit'
  import { decode, encode, rfc8949EncodeOptions } from 'cborg'
  import debounce from 'debounce'
  import { onMount } from 'svelte'

  let {
    auction,
    stateInfo,
    myInfo = $bindable(),
    onMyInfoChange
  }: {
    auction: AuctionService
    stateInfo: StateInfo
    myInfo: UserInfo
    onMyInfoChange?: (next: UserInfo) => void
  } = $props()

  const tokenInfo = $derived<TokenInfo>({
    name: stateInfo.token_name,
    symbol: stateInfo.token_symbol,
    decimals: stateInfo.token_decimals,
    fee: 0n,
    one: 10n ** BigInt(stateInfo.token_decimals),
    logo: stateInfo.token_logo_url,
    address: stateInfo.token
  })
  const currencyInfo = $derived<TokenInfo>({
    name: stateInfo.currency_name,
    symbol: stateInfo.currency_symbol,
    decimals: stateInfo.currency_decimals,
    fee: 0n,
    one: 10n ** BigInt(stateInfo.currency_decimals),
    logo: stateInfo.currency_logo_url,
    address: stateInfo.currency
  })
  const tokenDisplay = $derived.by(() => new TokenDisplay(tokenInfo, 0n))
  const currencyDisplay = $derived.by(() => new TokenDisplay(currencyInfo, 0n))

  let myDeposits = $state<DepositTxInfo[]>([])
  let myWithdraws = $state<WithdrawTxInfo[]>([])

  let activeTab = $state<'deposit' | 'withdraw'>('deposit')
  let depositMethod = $state<'1paying' | 'manual'>('1paying')
  let withdrawType = $state<'currency' | 'token'>('currency')
  let historyTab = $state<'deposits' | 'withdraws'>('deposits')

  const principal = $derived<string>(authStore.identity.getPrincipal().toText())
  const defaultBoundAddress = $derived.by(() => myInfo.bound_addresses[0] || '')

  let withdrawCurrencyRecipient = $state('')
  let withdrawTokenRecipient = $state('')

  // Deposit inputs
  let depositAmount = $state('')
  let manualDepositTxid = $state('')
  let manualDepositSender = $state('')

  async function refreshAll() {
    const [infoRes, depRes, wdRes] = await Promise.all([
      auction.my_info(),
      auction.my_deposits(),
      auction.my_withdraws()
    ])

    const _myInfo = unwrapResult(infoRes, 'failed to fetch user info')
    myInfo = _myInfo
    onMyInfoChange?.(_myInfo)
    myDeposits = unwrapResult(depRes, 'failed to fetch deposits')
    myWithdraws = unwrapResult(wdRes, 'failed to fetch withdraws')
  }

  let isBinding = $state(false)
  function bindAddress() {
    if (isBinding) return
    isBinding = true

    toastRun(async (signal) => {
      const x402 = await auction.x402_payment(10000n, true)
      const res = unwrapResult(
        x402,
        'failed to create binding payment'
      ) as X402PaymentOutput & { paymentRequired: PaymentRequired }
      res.paymentRequired = decode(res.x402 as Uint8Array)
      const bindReq = await payingKit.getPayUrl(res.paymentRequired)
      const payWindow = window.open(bindReq.payUrl, '1paying-checkout')
      const paymentPayload = await payingKit.waitForPaymentPayload(
        bindReq.txid,
        { signal }
      )

      if (payWindow && !payWindow.closed) {
        payWindow.close()
      }

      const verify = await x402Verify({
        paymentRequirements: res.paymentRequired
          .accepts[0] as PaymentRequirements,
        paymentPayload: JSON.parse(base64ToString(paymentPayload)),
        nonce: res.nonce
      })

      const bindRes = await auction.x402_bind_address({
        result: encode(verify.result, rfc8949EncodeOptions),
        signature: base64ToBytes(verify.signature),
        timestamp: res.timestamp
      })
      unwrapResult(bindRes, 'failed to bind address')

      await sleep(2000)
      await refreshAll()
      triggerToast({ type: 'success', message: 'Address has been bound' })
    }).finally(() => {
      isBinding = false
    })
  }

  const dRefreshDepositReq = debounce(refreshDepositReq, 500, {
    immediate: false
  })

  const [depositAmountUnits, depositAmountErr] = $derived.by(() => {
    if (!depositAmount.trim()) return [0n, '']
    try {
      const amt = parseUnits(depositAmount, currencyInfo.decimals)
      if (amt <= 0n) return [0n, 'Invalid amount']
      dRefreshDepositReq()
      return [amt, '']
    } catch {
      return [0n, 'Invalid amount']
    }
  })

  let x402Payment = $state<
    (X402PaymentOutput & { paymentRequired: PaymentRequired }) | null
  >(null)
  let depositReq = $state<{
    payUrl: string
    txid: string
  } | null>(null)
  function refreshDepositReq() {
    toastRun(async () => {
      const x402 = await auction.x402_payment(depositAmountUnits, false)
      const res = unwrapResult(
        x402,
        'failed to estimate x402 payment'
      ) as X402PaymentOutput & { paymentRequired: PaymentRequired }
      res.paymentRequired = decode(res.x402 as Uint8Array)
      x402Payment = res
      depositReq = await payingKit.getPayUrl(res.paymentRequired)
    })
  }

  let isPaying = $state(false)
  function depositVia1Paying() {
    if (depositAmountUnits <= 0n) {
      throw new Error(depositAmountErr || 'Invalid amount')
    }
    if (isPaying || !depositReq) return
    isPaying = true
    const payWindow = window.open(depositReq.payUrl, '1paying-checkout')
    toastRun(async (signal) => {
      if (!depositReq || !x402Payment) return

      const paymentPayload = await payingKit.waitForPaymentPayload(
        depositReq.txid,
        { signal }
      )

      if (payWindow && !payWindow.closed) {
        // Close the payment window if it's still open
        // We can not reuse the window for another payment due to browser security policies
        // Error: Blocked a frame with origin "https://1paying-coffee.zensh.workers.dev" from accessing a cross-origin frame.
        payWindow.close()
      }

      const settled = await x402Settle({
        paymentRequirements: x402Payment.paymentRequired
          .accepts[0] as PaymentRequirements,
        paymentPayload: JSON.parse(base64ToString(paymentPayload)),
        nonce: x402Payment.nonce
      })

      const depositRes = await auction.x402_deposit_currency({
        result: encode(settled.result, rfc8949EncodeOptions),
        signature: base64ToBytes(settled.signature),
        timestamp: x402Payment.timestamp
      })

      unwrapResult(depositRes, 'failed to deposit currency via 1Paying')
      triggerToast({ type: 'success', message: 'Currency has been deposited' })
      depositAmount = ''
      await payingKit.submitSettleResult(
        depositReq.txid,
        settled.result.settleResponse
      )
      await sleep(2000)
      await refreshAll()
    }).finally(() => {
      isPaying = false
      if (payWindow && !payWindow.closed) {
        payWindow.close()
      }
    })
  }

  let isDepositingManually = $state(false)
  function depositManually() {
    if (isDepositingManually) return
    isDepositingManually = true
    toastRun(async () => {
      const txid = manualDepositTxid.trim()
      const sender = manualDepositSender.trim()
      if (!txid) throw new Error('Please fill in txid')
      if (!sender) throw new Error('Please fill in sender')
      const res = await auction.deposit_currency({ txid, sender })
      unwrapResult(res, 'deposit currency failed')
      triggerToast({ type: 'success', message: 'Currency has been deposited' })
      await sleep(2000)
      await refreshAll()
    }).finally(() => {
      isDepositingManually = false
      manualDepositTxid = ''
      manualDepositSender = ''
    })
  }

  let isWithdrawingCurrency = $state(false)
  function withdrawCurrency() {
    if (isWithdrawingCurrency) return
    isWithdrawingCurrency = true
    toastRun(async () => {
      const recipient = withdrawCurrencyRecipient.trim()
      if (!recipient) throw new Error('Please fill in recipient')
      const res = await auction.withdraw_currency({ recipient })
      unwrapResult(res, 'withdraw currency failed')
      triggerToast({ type: 'success', message: 'Currency has been withdrawn' })
      await sleep(2000)
      await refreshAll()
    }).finally(() => {
      isWithdrawingCurrency = false
      withdrawCurrencyRecipient = ''
    })
  }

  let isWithdrawingToken = $state(false)
  function withdrawToken() {
    if (isWithdrawingToken) return

    isWithdrawingToken = true
    toastRun(async () => {
      const recipient = withdrawTokenRecipient.trim()
      if (!recipient) throw new Error('Please fill in recipient')
      const res = await auction.withdraw_token({ recipient })
      unwrapResult(res, 'withdraw token failed')
      triggerToast({ type: 'success', message: 'Token has been withdrawn' })
      await sleep(2000)
      await refreshAll()
    }).finally(() => {
      isWithdrawingToken = false
      withdrawTokenRecipient = ''
    })
  }

  onMount(() => {
    return toastRun(refreshAll).abort
  })
</script>

<div class="relative space-y-6">
  <div class="flex flex-col gap-4">
    <div
      class="border-border-subtle flex items-center justify-between border-b pb-2"
    >
      <div class="flex gap-6">
        <button
          class="relative py-2 text-sm font-bold tracking-wide uppercase transition-colors {activeTab ===
          'deposit'
            ? 'text-foreground'
            : 'text-muted hover:text-foreground'}"
          onclick={() => (activeTab = 'deposit')}
        >
          Deposit
          {#if activeTab === 'deposit'}
            <div class="bg-foreground absolute -bottom-2 left-0 h-0.5 w-full"
            ></div>
          {/if}
        </button>
        <button
          class="relative py-2 text-sm font-bold tracking-wide uppercase transition-colors {activeTab ===
          'withdraw'
            ? 'text-foreground'
            : 'text-muted hover:text-foreground'}"
          onclick={() => (activeTab = 'withdraw')}
        >
          Withdraw
          {#if activeTab === 'withdraw'}
            <div class="bg-foreground absolute -bottom-2 left-0 h-0.5 w-full"
            ></div>
          {/if}
        </button>
      </div>

      <div class="flex items-center gap-3">
        <div class="flex items-center gap-1.5 text-xs">
          <span class="text-muted font-semibold uppercase">Principal</span>
          <span class="font-mono">{pruneAddress(principal, true)}</span>
          <TextClipboardButton
            value={principal}
            class="text-muted hover:text-foreground transition *:size-4"
          />
        </div>
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-xs">
      <div class="text-muted font-semibold uppercase">Bound Addresses</div>
      <div class="flex flex-wrap items-center gap-2">
        {#each myInfo.bound_addresses as a (a)}
          <div class="bg-surface flex items-center gap-1 rounded-md px-2 py-1">
            <span class="font-mono">{pruneAddress(a, true)}</span>
            <TextClipboardButton
              value={a}
              class="text-muted hover:text-foreground transition *:size-3.5"
            />
          </div>
        {/each}
        {#if myInfo.bound_addresses.length === 0}
          <span class="text-muted italic">No bound addresses.</span>
        {/if}
        <Button
          class="text-muted hover:text-foreground border-border-subtle hover:border-foreground rounded-md border px-2 py-1 text-[10px] font-bold uppercase transition-colors"
          onclick={bindAddress}
          isLoading={isBinding}
        >
          Bind via 1Paying
        </Button>
      </div>
    </div>
  </div>

  <section class="min-h-80 space-y-6">
    {#if activeTab === 'deposit'}
      <div class="space-y-4">
        <div class="flex gap-2">
          <button
            class="rounded-full px-4 py-1.5 text-xs font-semibold transition-colors {depositMethod ===
            '1paying'
              ? 'bg-foreground text-background'
              : 'bg-surface text-muted hover:text-foreground'}"
            onclick={() => (depositMethod = '1paying')}
          >
            via 1Paying
          </button>
          <button
            class="rounded-full px-4 py-1.5 text-xs font-semibold transition-colors {depositMethod ===
            'manual'
              ? 'bg-foreground text-background'
              : 'bg-surface text-muted hover:text-foreground'}"
            onclick={() => (depositMethod = 'manual')}
          >
            Manual
          </button>
        </div>

        {#if depositMethod === '1paying'}
          <div
            class="border-border-subtle bg-card max-w-md rounded-2xl border p-6 shadow-sm"
          >
            <div class="mb-4 text-sm font-semibold"
              >Deposit Currency via 1Paying</div
            >
            <div class="space-y-4">
              <div class="space-y-1.5">
                <label
                  class="text-muted text-[10px] font-bold uppercase"
                  for="deposit-amount"
                >
                  Amount ({currencyInfo.symbol})
                </label>
                <input
                  id="deposit-amount"
                  class="border-border-subtle bg-surface transition-focus focus:border-foreground w-full rounded-xl border px-4 py-3 text-sm focus:outline-none"
                  placeholder="0.00"
                  bind:value={depositAmount}
                  inputmode="decimal"
                />
                {#if depositAmountErr}
                  <div class="text-muted text-xs">{depositAmountErr}</div>
                {/if}
              </div>
              <Button
                class="bg-foreground text-background w-full rounded-xl py-3 text-sm font-bold tracking-wide uppercase transition-opacity hover:opacity-90 disabled:opacity-50"
                onclick={depositVia1Paying}
                disabled={depositAmountUnits == 0n ||
                  !!depositAmountErr ||
                  !depositReq}
                isLoading={isPaying}
              >
                Pay & Deposit
              </Button>
            </div>
          </div>
        {:else}
          <div
            class="border-border-subtle bg-card max-w-md rounded-2xl border p-6 shadow-sm"
          >
            <div class="mb-4 text-sm font-semibold">Manual Deposit</div>
            <p class="text-muted mb-4 text-xs">
              Submit an on-chain deposit record for reconciliation.
            </p>
            <div class="space-y-4">
              <div class="space-y-1.5">
                <label
                  class="text-muted text-[10px] font-bold uppercase"
                  for="manual-txid"
                >
                  Transaction ID
                </label>
                <input
                  id="manual-txid"
                  class="border-border-subtle bg-surface transition-focus focus:border-foreground w-full rounded-xl border px-4 py-3 text-sm focus:outline-none"
                  placeholder="Enter txid"
                  bind:value={manualDepositTxid}
                />
              </div>
              <div class="space-y-1.5">
                <label
                  class="text-muted text-[10px] font-bold uppercase"
                  for="manual-sender"
                >
                  Sender Address
                </label>
                <input
                  id="manual-sender"
                  class="border-border-subtle bg-surface transition-focus focus:border-foreground w-full rounded-xl border px-4 py-3 text-sm focus:outline-none"
                  placeholder="Enter sender address"
                  bind:value={manualDepositSender}
                  onfocus={() => (manualDepositSender = defaultBoundAddress)}
                />
              </div>
              <Button
                class="bg-foreground text-background w-full rounded-xl py-3 text-sm font-bold tracking-wide uppercase transition-opacity hover:opacity-90 disabled:opacity-50"
                onclick={depositManually}
                disabled={!manualDepositTxid || !manualDepositSender}
                isLoading={isDepositingManually}
              >
                Submit Deposit
              </Button>
            </div>
          </div>
        {/if}
      </div>
    {:else}
      <div class="space-y-4">
        <div class="flex gap-2">
          <button
            class="rounded-full px-4 py-1.5 text-xs font-semibold transition-colors {withdrawType ===
            'currency'
              ? 'bg-foreground text-background'
              : 'bg-surface text-muted hover:text-foreground'}"
            onclick={() => (withdrawType = 'currency')}
          >
            {currencyInfo.symbol}
          </button>
          <button
            class="rounded-full px-4 py-1.5 text-xs font-semibold transition-colors {withdrawType ===
            'token'
              ? 'bg-foreground text-background'
              : 'bg-surface text-muted hover:text-foreground'}"
            onclick={() => (withdrawType = 'token')}
          >
            {tokenInfo.symbol}
          </button>
        </div>

        {#if withdrawType === 'currency'}
          <div
            class="border-border-subtle bg-card max-w-md rounded-2xl border p-6 shadow-sm"
          >
            <div class="mb-4 flex items-center justify-between">
              <span class="text-sm font-semibold"
                >Withdraw {currencyInfo.symbol}</span
              >
              <span class="text-muted text-xs">
                Balance: {currencyDisplay.displayValue(myInfo.currency_amount)}
              </span>
            </div>
            <div class="space-y-4">
              <div class="space-y-1.5">
                <label
                  class="text-muted text-[10px] font-bold uppercase"
                  for="withdraw-currency-recipient"
                >
                  Recipient Address
                </label>
                <input
                  id="withdraw-currency-recipient"
                  class="border-border-subtle bg-surface transition-focus focus:border-foreground w-full rounded-xl border px-4 py-3 text-sm focus:outline-none"
                  placeholder="Enter recipient address"
                  bind:value={withdrawCurrencyRecipient}
                  onfocus={() =>
                    (withdrawCurrencyRecipient = defaultBoundAddress)}
                />
              </div>
              <Button
                class="bg-foreground text-background w-full rounded-xl py-3 text-sm font-bold tracking-wide uppercase transition-opacity hover:opacity-90 disabled:opacity-50"
                onclick={withdrawCurrency}
                disabled={myInfo.currency_amount == 0n ||
                  !withdrawCurrencyRecipient}
                isLoading={isWithdrawingCurrency}
              >
                Withdraw {currencyInfo.symbol}
              </Button>
            </div>
          </div>
        {:else}
          <div
            class="border-border-subtle bg-card max-w-md rounded-2xl border p-6 shadow-sm"
          >
            <div class="mb-4 flex items-center justify-between">
              <span class="text-sm font-semibold"
                >Withdraw {tokenInfo.symbol}</span
              >
              <span class="text-muted text-xs">
                Balance: {tokenDisplay.displayValue(myInfo.token_amount)}
              </span>
            </div>
            <div class="space-y-4">
              <div class="space-y-1.5">
                <label
                  class="text-muted text-[10px] font-bold uppercase"
                  for="withdraw-token-recipient"
                >
                  Recipient Address
                </label>
                <input
                  id="withdraw-token-recipient"
                  class="border-border-subtle bg-surface transition-focus focus:border-foreground w-full rounded-xl border px-4 py-3 text-sm focus:outline-none"
                  placeholder="Enter recipient address"
                  bind:value={withdrawTokenRecipient}
                  onfocus={() => (withdrawTokenRecipient = defaultBoundAddress)}
                />
              </div>
              <Button
                class="bg-foreground text-background w-full rounded-xl py-3 text-sm font-bold tracking-wide uppercase transition-opacity hover:opacity-90 disabled:opacity-50"
                onclick={withdrawToken}
                disabled={myInfo.token_amount == 0n || !withdrawTokenRecipient}
                isLoading={isWithdrawingToken}
              >
                Withdraw {tokenInfo.symbol}
              </Button>
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </section>

  <div class="space-y-4 pt-4">
    <div
      class="border-border-subtle flex items-center justify-between border-b pb-2"
    >
      <div class="flex gap-6">
        <button
          class="relative py-2 text-xs font-bold tracking-wide uppercase transition-colors {historyTab ===
          'deposits'
            ? 'text-foreground'
            : 'text-muted hover:text-foreground'}"
          onclick={() => (historyTab = 'deposits')}
        >
          Deposits
          {#if historyTab === 'deposits'}
            <div class="bg-foreground absolute -bottom-2 left-0 h-0.5 w-full"
            ></div>
          {/if}
        </button>
        <button
          class="relative py-2 text-xs font-bold tracking-wide uppercase transition-colors {historyTab ===
          'withdraws'
            ? 'text-foreground'
            : 'text-muted hover:text-foreground'}"
          onclick={() => (historyTab = 'withdraws')}
        >
          Withdraws
          {#if historyTab === 'withdraws'}
            <div class="bg-foreground absolute -bottom-2 left-0 h-0.5 w-full"
            ></div>
          {/if}
        </button>
      </div>
    </div>

    <div class="min-h-[200px]">
      {#if historyTab === 'deposits'}
        {#if myDeposits.length === 0}
          <div class="text-muted py-8 text-center text-sm italic"
            >No deposit records yet.</div
          >
        {:else}
          <div
            class="border-border-subtle divide-border-subtle divide-y overflow-hidden rounded-xl border"
          >
            {#each myDeposits as d (d.txid)}
              {@const txUrl = getTxUrl(stateInfo.chain, d.txid)}
              <div
                class="bg-card hover:bg-surface px-4 py-3 text-xs transition-colors"
              >
                <div class="flex items-center justify-between">
                  <div class="font-semibold">
                    {#if txUrl}
                      <a
                        class="hover:text-foreground inline-flex items-center gap-1 underline decoration-dotted underline-offset-2"
                        href={txUrl}
                        target="_blank"
                        rel="noreferrer"
                      >
                        {pruneAddress(d.txid, true)}
                        <ArrowRightUpLine class="size-3" />
                      </a>
                    {:else}
                      {pruneAddress(d.txid, true)}
                    {/if}
                  </div>
                  <div class="text-muted">{formatDatetime(d.timestamp)}</div>
                </div>
                <div class="text-muted mt-1.5 flex items-center gap-2">
                  <span class="text-foreground font-medium">
                    {currencyDisplay.displayValue(d.amount)}
                    {currencyInfo.symbol}
                  </span>
                  <span>·</span>
                  <span>sender: {pruneAddress(d.sender)}</span>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      {:else if myWithdraws.length === 0}
        <div class="text-muted py-8 text-center text-sm italic"
          >No withdraw records yet.</div
        >
      {:else}
        <div
          class="border-border-subtle divide-border-subtle divide-y overflow-hidden rounded-xl border"
        >
          {#each myWithdraws as w (w.id)}
            {@const txUrl = getTxUrl(stateInfo.chain, w.txid)}
            <div
              class="bg-card hover:bg-surface px-4 py-3 text-xs transition-colors"
            >
              <div class="flex items-center justify-between">
                <div class="font-semibold">#{w.id.toString()}</div>
                <div class="text-muted">{formatDatetime(w.timestamp)}</div>
              </div>
              <div
                class="text-muted mt-1.5 flex flex-wrap items-center gap-x-2 gap-y-1"
              >
                <span class="text-foreground font-medium">
                  {#if w.kind == 0}
                    {currencyDisplay.displayValue(w.amount)}
                    {currencyInfo.symbol}
                  {:else}
                    {tokenDisplay.displayValue(w.amount)}
                    {tokenInfo.symbol}
                  {/if}
                </span>
                <span>·</span>
                <span>recipient: {pruneAddress(w.recipient)}</span>
                <span>·</span>
                {#if txUrl}
                  <a
                    class="hover:text-foreground inline-flex items-center gap-1 underline decoration-dotted underline-offset-2"
                    href={txUrl}
                    target="_blank"
                    rel="noreferrer"
                  >
                    {pruneAddress(w.txid, true)}
                    <ArrowRightUpLine class="size-3" />
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
</div>
