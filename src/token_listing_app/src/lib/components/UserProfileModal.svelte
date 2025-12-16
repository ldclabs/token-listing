<script lang="ts">
  import LogoutCircleRLine from '$lib/icons/logout-circle-r-line.svelte'
  import { authStore } from '$lib/stores/auth.svelte'
  import TextClipboardButton from '$lib/ui/TextClipboardButton.svelte'
  import { pruneAddress } from '$lib/utils/helper'

  const address = $derived<string>(authStore.identity.getPrincipal().toText())
</script>

<div class="relative">
  <div class="flex flex-col gap-1">
    <div class="flex flex-row items-center gap-2 text-sm text-slate-500">
      <span class="font-medium tracking-wide uppercase">Your address:</span>
      <span>{pruneAddress(address, true)}</span>
      <TextClipboardButton
        value={address}
        class="text-slate-400 transition *:size-5 hover:text-slate-600"
      />
    </div>
  </div>

  <hr class="my-4 border-zinc-200" />

  <button
    type="button"
    class="mx-auto flex w-fit items-center gap-2 px-4 py-2 text-amber-600 hover:text-amber-800"
    onclick={() => authStore.logout()}
  >
    <LogoutCircleRLine class="size-4" /><span>Logout</span>
  </button>
</div>
