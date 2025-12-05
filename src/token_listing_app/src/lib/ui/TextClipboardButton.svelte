<script lang="ts">
  import CheckLine from '$lib/icons/check-line.svelte'
  import FileCopyLine from '$lib/icons/file-copy-line.svelte'
  import { copyTextToClipboard } from '$lib/utils/window'

  let {
    value,
    class: className = ''
  }: {
    value: string
    class?: string
  } = $props()

  let copiedClass = $state('')

  async function onCopyHandler() {
    const ok = await copyTextToClipboard(value)
    if (ok) {
      copiedClass = '!text-green-500 '
      setTimeout(() => {
        copiedClass = ''
      }, 5000)
    }
  }
</script>

<button
  class={copiedClass + className}
  onclick={onCopyHandler}
  disabled={copiedClass != ''}
>
  {#if copiedClass != ''}
    <CheckLine />
  {:else}
    <FileCopyLine />
  {/if}
</button>
