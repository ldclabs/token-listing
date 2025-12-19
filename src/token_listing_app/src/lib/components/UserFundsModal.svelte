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
      const x402 = await auction.x402_payment(1n, true)
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
  <div class="flex flex-col gap-2">
    <div class="text-muted text-xs font-semibold tracking-wide uppercase">
      Account
    </div>
    <div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm">
      <div class="font-semibold">Principal</div>
      <span>{pruneAddress(principal, true)}</span>
      <TextClipboardButton
        value={principal}
        class="text-slate-400 transition *:size-5 hover:text-slate-600"
      />
    </div>

    <div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm">
      <div class="font-semibold">Bound Addresses</div>
      <ul class="">
        {#each myInfo.bound_addresses as a (a)}
          <li class="mr-2 inline-flex items-center gap-1">
            <span>{pruneAddress(a, true)}</span>
            <TextClipboardButton
              value={a}
              class="text-slate-400 transition *:size-5 hover:text-slate-600"
            />
          </li>
        {/each}
        {#if myInfo.bound_addresses.length === 0}
          <li class="text-muted">No bound addresses.</li>
        {/if}
      </ul>
      <Button
        class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-3 py-1 text-xs"
        onclick={bindAddress}
        isLoading={isBinding}
      >
        Bind address via 1Paying
      </Button>
    </div>
  </div>

  <section class="space-y-4">
    <div class="">
      <div class="text-muted text-xs font-semibold tracking-wide uppercase">
        Deposit Currency
      </div>

      <div class="mt-3 grid gap-3 sm:grid-cols-2">
        <div class="border-border-subtle bg-card rounded-xl border p-4 shadow">
          <div class="text-xs font-semibold">Deposit via 1Paying</div>
          <div class="mt-3 space-y-2">
            <input
              class="border-border-subtle bg-card w-full rounded-lg border px-3 py-2 text-xs"
              placeholder={`amount (${currencyInfo.symbol})`}
              bind:value={depositAmount}
              inputmode="decimal"
            />
            {#if depositAmountErr}
              <div class="text-muted text-xs">{depositAmountErr}</div>
            {/if}
            <Button
              class="border-border-subtle text-muted hover:border-foreground hover:text-foreground w-full rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
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

        <div class="border-border-subtle bg-card rounded-xl border p-4 shadow">
          <div class="text-xs font-semibold">Manual Deposit</div>
          <div class="text-muted mt-1 text-xs">
            Submit an on-chain deposit record for reconciliation.
          </div>
          <div class="mt-3 space-y-2">
            <input
              class="border-border-subtle bg-card w-full rounded-lg border px-3 py-2 text-xs"
              placeholder="txid"
              bind:value={manualDepositTxid}
            />
            <input
              class="border-border-subtle bg-card w-full rounded-lg border px-3 py-2 text-xs"
              placeholder="sender"
              bind:value={manualDepositSender}
              onfocus={() => (manualDepositSender = defaultBoundAddress)}
            />
            <Button
              class="border-border-subtle text-muted hover:border-foreground hover:text-foreground w-full rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
              onclick={depositManually}
              disabled={!manualDepositTxid || !manualDepositSender}
              isLoading={isDepositingManually}
            >
              Submit Deposit
            </Button>
          </div>
        </div>
      </div>
    </div>

    <div class="">
      <div class="text-muted text-xs font-semibold tracking-wide uppercase">
        Balances
      </div>
      <div class="mt-3 grid gap-3 sm:grid-cols-2">
        <div class="border-border-subtle bg-card rounded-xl border p-4 shadow">
          <div class="flex items-center justify-between">
            <span class="text-xs font-semibold">Currency</span>
            <span class="text-xs">
              {currencyDisplay.displayValue(myInfo.currency_amount)}
              {currencyInfo.symbol}
            </span>
          </div>
          <div class="mt-3 space-y-2">
            <input
              class="border-border-subtle bg-card w-full rounded-lg border px-3 py-2 text-xs"
              placeholder="recipient"
              bind:value={withdrawCurrencyRecipient}
              onfocus={() => (withdrawCurrencyRecipient = defaultBoundAddress)}
            />
            <Button
              class="border-border-subtle text-muted hover:border-foreground hover:text-foreground w-full rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
              onclick={withdrawCurrency}
              disabled={myInfo.currency_amount == 0n ||
                !withdrawCurrencyRecipient}
              isLoading={isWithdrawingCurrency}
            >
              Withdraw
            </Button>
          </div>
        </div>

        <div class="border-border-subtle bg-card rounded-xl border p-4 shadow">
          <div class="flex items-center justify-between">
            <span class="text-xs font-semibold">Token</span>
            <span class="text-xs">
              {tokenDisplay.displayValue(myInfo.token_amount)}
              {tokenInfo.symbol}
            </span>
          </div>
          <div class="mt-3 space-y-2">
            <input
              class="border-border-subtle bg-card w-full rounded-lg border px-3 py-2 text-xs"
              placeholder="recipient"
              bind:value={withdrawTokenRecipient}
              onfocus={() => (withdrawTokenRecipient = defaultBoundAddress)}
            />
            <Button
              class="border-border-subtle text-muted hover:border-foreground hover:text-foreground w-full rounded-full border px-3 py-2 text-xs font-semibold tracking-wide uppercase"
              onclick={withdrawToken}
              disabled={myInfo.token_amount == 0n || !withdrawTokenRecipient}
              isLoading={isWithdrawingToken}
            >
              Withdraw
            </Button>
          </div>
        </div>
      </div>
    </div>

    <div class="grid gap-4 lg:grid-cols-2">
      <div>
        <div
          class="text-muted mb-2 text-xs font-semibold tracking-wide uppercase"
        >
          Deposits
        </div>
        {#if myDeposits.length === 0}
          <div class="text-muted text-sm">No records yet.</div>
        {:else}
          <div class="border-border-subtle overflow-hidden rounded-xl border">
            {#each myDeposits as d (d.txid)}
              {@const txUrl = getTxUrl(stateInfo.chain, d.txid)}
              <div
                class="border-border-subtle border-t px-3 py-2 text-xs first:border-t-0"
              >
                <div class="flex items-center justify-between">
                  <div class="font-semibold">
                    {#if txUrl}
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
                  <div class="text-muted">{formatDatetime(d.timestamp)}</div>
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
        >
          Withdraws
        </div>
        {#if myWithdraws.length === 0}
          <div class="text-muted text-sm">No records yet.</div>
        {:else}
          <div class="border-border-subtle overflow-hidden rounded-xl border">
            {#each myWithdraws as w (w.id)}
              {@const txUrl = getTxUrl(stateInfo.chain, w.txid)}
              <div
                class="border-border-subtle border-t px-3 py-2 text-xs first:border-t-0"
              >
                <div class="flex items-center justify-between">
                  <div class="font-semibold">#{w.id.toString()}</div>
                  <div class="text-muted">{formatDatetime(w.timestamp)}</div>
                </div>
                <div class="text-muted mt-1">
                  {#if w.kind == 0}
                    <span>
                      {currencyDisplay.displayValue(w.amount)}
                      {currencyInfo.symbol}
                    </span>
                  {:else}
                    <span>
                      {tokenDisplay.displayValue(w.amount)}
                      {tokenInfo.symbol}
                    </span>
                  {/if}
                  <span>{` · recipient: ${pruneAddress(w.recipient)} · `}</span>
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
</div>
