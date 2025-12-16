<script lang="ts">
  import type { StateInfo } from '$declarations/ic_auction/ic_auction.did'
  import ArrowRightUpLine from '$lib/icons/arrow-right-up-line.svelte'
  import TextClipboardButton from '$lib/ui/TextClipboardButton.svelte'
  import { pruneAddress } from '$lib/utils/helper'

  let {
    stateInfo
  }: {
    stateInfo: StateInfo
  } = $props()

  const canister = $derived(stateInfo.icp_address.toString())
</script>

<div class="relative">
  <div
    class="border-border-subtle bg-surface space-y-2 rounded-lg border p-3 text-sm"
  >
    <div class="flex flex-row items-center gap-2">
      <span class="font-medium tracking-wide uppercase">Smart Contract:</span>
      <a
        class="text-muted hover:border-foreground hover:text-foreground inline-flex items-center gap-1"
        href={`https://dashboard.internetcomputer.org/canister/${canister}`}
        target="_blank"
        rel="noreferrer"
      >
        {pruneAddress(canister)}
        <ArrowRightUpLine class="h-4 w-4" />
      </a>
    </div>
    <div class="flex flex-row items-center gap-2">
      <span class="font-medium tracking-wide uppercase">ICP Address:</span>
      <span>{pruneAddress(canister)}</span>
      <TextClipboardButton
        value={canister}
        class="text-slate-400 transition *:size-5 hover:text-slate-600"
      />
    </div>
    <div class="flex flex-row items-center gap-2">
      <span class="font-medium tracking-wide uppercase">SOL Address:</span>
      <span>{pruneAddress(stateInfo.sol_address)}</span>
      <TextClipboardButton
        value={stateInfo.sol_address}
        class="text-slate-400 transition *:size-5 hover:text-slate-600"
      />
    </div>
    <div class="flex flex-row items-center gap-2">
      <span class="font-medium tracking-wide uppercase">EVM Address:</span>
      <span>{pruneAddress(stateInfo.evm_address)}</span>
      <TextClipboardButton
        value={stateInfo.evm_address}
        class="text-slate-400 transition *:size-5 hover:text-slate-600"
      />
    </div>
  </div>
</div>
