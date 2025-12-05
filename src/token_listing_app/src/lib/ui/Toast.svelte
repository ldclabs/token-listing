<script lang="ts">
  import CheckboxCircleFill from '$lib/icons/checkbox-circle-fill.svelte'
  import CloseCircleFill from '$lib/icons/close-circle-fill.svelte'
  import CloseCircleLine from '$lib/icons/close-line.svelte'
  import InfoCircleFill from '$lib/icons/information-2-fill.svelte'
  import { toastStore } from '$lib/stores/toast.svelte.ts'
  import { slide } from 'svelte/transition'

  const typeClasses = {
    success: 'text-green-500',
    error: 'text-red-600',
    info: 'text-cyan-500'
  }

  function init(ele: HTMLDivElement) {
    const checkState = () => {
      if (ele.childElementCount > 0) {
        try {
          // 重新打开以确保位于 Top Layer 顶部
          try {
            ele.hidePopover()
          } catch {}
          ele.showPopover()
        } catch {}
      } else {
        try {
          ele.hidePopover()
        } catch {}
      }
    }
    // 初始检查
    checkState()

    const observer = new MutationObserver(checkState)
    observer.observe(ele, { childList: true })

    return () => observer.disconnect()
  }
</script>

{#snippet icon(type: string)}
  {#if type === 'success'}
    <CheckboxCircleFill />
  {:else if type === 'error'}
    <CloseCircleFill />
  {:else if type === 'info'}
    <InfoCircleFill />
  {/if}
{/snippet}

<div
  {@attach init}
  popover="manual"
  class="pointer-events-none fixed top-auto right-4 bottom-4 left-auto z-[2147483647] m-0 flex max-w-sm flex-col items-end gap-2 border-0 bg-transparent p-0 outline-none"
>
  {#each toastStore as toast (toast.id)}
    <div
      class="pointer-events-auto grid items-center gap-3 rounded-xl bg-white/90 p-4 text-black/90 shadow-md {toast.dismissable
        ? 'grid-cols-[auto_1fr_auto]'
        : 'grid-cols-[auto_1fr]'}"
      transition:slide={{ duration: 250 }}
      role="alert"
      aria-live="assertive"
    >
      <div
        class="flex flex-shrink-0 items-center justify-center rounded bg-black/10 p-1 *:size-6 {typeClasses[
          toast.type
        ]}"
      >
        {@render icon(toast.type)}
      </div>
      <div class="break-all">
        {#if toast.message}
          {toast.message}
        {:else if toast.content}
          {@render toast.content()}
        {/if}
      </div>
      {#if toast.dismissable}
        <button
          class="*:size-6 {typeClasses[
            toast.type
          ]} cursor-pointer rounded-full p-1 transition-all duration-300 hover:rotate-180 hover:bg-black/10"
          onclick={toast.onclose}
        >
          <CloseCircleLine />
        </button>
      {/if}
    </div>
  {/each}
</div>
