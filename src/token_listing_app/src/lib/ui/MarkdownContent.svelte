<script lang="ts">
  import ArrowDownSLine from '$lib/icons/arrow-down-s-line.svelte'
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
    <div class="relative">
      <div
        class={`md-content w-full text-pretty wrap-break-word ${!isExpanded && isLong ? 'cca-desc-clamp' : ''}`}
      >
        {@html result[0]}
      </div>

      {#if !isExpanded && isLong}
        <div
          class="from-background via-background/90 pointer-events-none absolute bottom-0 left-0 flex h-16 w-full items-end justify-center bg-linear-to-t to-transparent pb-1"
        >
          <button
            class="bg-background pointer-events-auto flex items-center gap-1 rounded-full px-4 py-1 text-[10px] font-bold tracking-widest text-indigo-500 shadow-md ring-1 ring-indigo-500/30 transition-all hover:bg-indigo-500 hover:text-white"
            onclick={() => (isExpanded = !isExpanded)}
            type="button"
          >
            SHOW MORE
            <ArrowDownSLine class="h-4 w-4" />
          </button>
        </div>
      {/if}
    </div>

    {#if isExpanded && isLong}
      <div class="flex justify-end">
        <button
          class="inline-flex items-center gap-1 text-xs font-semibold tracking-wide text-indigo-500 uppercase transition-colors hover:text-indigo-700"
          onclick={() => (isExpanded = !isExpanded)}
          type="button"
        >
          Show less
          <ArrowDownSLine
            class="h-4 w-4 rotate-180 transition-transform duration-200"
          />
        </button>
      </div>
    {/if}
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
