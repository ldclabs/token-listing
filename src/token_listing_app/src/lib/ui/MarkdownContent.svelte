<script lang="ts">
  import { renderMarkdown } from '$lib/utils/markdown'

  let { content }: { content: string } = $props()

  const result = $derived(renderMarkdown(content))

  $effect(() => {
    result[1]()
  })

  let isExpanded = $state(false)
  const isLong = $derived.by(() => {
    if (!content) return false
    return content.length > 280 || content.split('\n').length > 6
  })
</script>

{#if content}
  <div class="space-y-2">
    <div
      class={`md-content w-full text-pretty wrap-break-word ${!isExpanded && isLong ? 'cca-desc-clamp' : ''}`}
    >
      {@html result[0]}
    </div>
    <div class="flex justify-end">
      {#if isLong}
        <button
          class="inline-flex items-center gap-1 text-xs font-semibold tracking-wide text-indigo-500 uppercase hover:text-indigo-700"
          onclick={() => (isExpanded = !isExpanded)}
          type="button"
        >
          {isExpanded ? 'Show less' : 'Show more'}
        </button>
      {/if}
    </div>
  </div>
{:else}
  <div class="md-content w-full text-pretty wrap-break-word">—</div>
{/if}

<style>
  .cca-desc-clamp {
    overflow: hidden;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    line-clamp: 3;
    -webkit-line-clamp: 3;
  }
</style>
