<script lang="ts">
  import type { Snippet } from 'svelte'
  import { tick } from 'svelte'

  let {
    trigger,
    children,
    containerClass = '',
    contentClass = ''
  }: {
    trigger: Snippet<[]>
    children: Snippet<[]>
    containerClass?: string
    contentClass?: string
  } = $props()

  let visible = $state(false)
  let triggerWrapperEl = $state<HTMLElement | null>(null)
  let contentEl = $state<HTMLElement | null>(null)
  let contentStyle = $state('visibility: hidden; opacity: 0;')

  function show() {
    visible = true
  }

  function hide() {
    visible = false
    contentStyle = 'visibility: hidden; opacity: 0;'
  }

  function toggle() {
    if (visible) hide()
    else show()
  }

  function portal(node: HTMLElement) {
    document.body.appendChild(node)
    return () => {
      if (node.parentNode === document.body) document.body.removeChild(node)
    }
  }

  let containerEl = $state<HTMLElement | null>(null)

  $effect(() => {
    if (!visible || !containerEl) return

    const handleClickOutside = (event: MouseEvent) => {
      if (!containerEl?.contains(event.target as Node)) {
        hide()
      }
    }

    const handleScroll = () => {
      if (visible) hide()
    }

    document.addEventListener('click', handleClickOutside, true)
    window.addEventListener('scroll', handleScroll, true)
    return () => {
      document.removeEventListener('click', handleClickOutside, true)
      window.removeEventListener('scroll', handleScroll, true)
    }
  })

  $effect(() => {
    if (!visible) return

    if (visible && triggerWrapperEl) {
      tick().then(() => {
        if (!contentEl || !triggerWrapperEl) return

        const triggerRect = triggerWrapperEl.getBoundingClientRect()
        const contentRect = contentEl.getBoundingClientRect()

        const spaceAbove = triggerRect.top

        let top = 0
        let left =
          triggerRect.left + (triggerRect.width - contentRect.width) / 2

        // Keep within screen bounds
        left = Math.max(
          8,
          Math.min(left, window.innerWidth - contentRect.width - 8)
        )

        const isAbove = spaceAbove > contentRect.height + 12
        let arrowLeft = triggerRect.left - left + triggerRect.width / 2 - 6

        if (isAbove) {
          // Show above
          top = triggerRect.top - contentRect.height - 12
          contentEl.classList.add('is-above')
          contentEl.classList.remove('is-below')
        } else {
          // Show below
          top = triggerRect.bottom + 12
          contentEl.classList.add('is-below')
          contentEl.classList.remove('is-above')
        }

        contentStyle = `left: ${left}px; top: ${top}px; opacity: 1; visibility: visible; transform: scale(1); --arrow-left: ${arrowLeft}px;`
      })
    }
  })
</script>

<div
  class={'tooltip-container ' + containerClass}
  bind:this={containerEl}
  onmouseenter={show}
  onmouseleave={hide}
  onfocusin={show}
  onfocusout={hide}
  onclick={toggle}
  role="presentation"
>
  <div class="tooltip-trigger" bind:this={triggerWrapperEl}>
    {@render trigger()}
  </div>

  {#if visible}
    <div
      {@attach portal}
      class={'tooltip-content glass-effect ' + contentClass}
      bind:this={contentEl}
      style={contentStyle}
      role="tooltip"
    >
      <div class="tooltip-arrow"></div>
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .tooltip-container {
    position: relative;
    display: inline-block;
  }

  .tooltip-trigger {
    display: block;
    cursor: help;
  }

  .tooltip-content {
    position: fixed;
    z-index: 1000;
    pointer-events: none;
    transition:
      opacity 0.2s ease-out,
      transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
    transform: scale(0.95);
    transform-origin: center bottom;
  }

  .tooltip-content.is-below {
    transform-origin: center top;
  }

  .tooltip-arrow {
    position: absolute;
    width: 12px;
    height: 12px;
    background: inherit;
    border: inherit;
    border-right: 0;
    border-bottom: 0;
    left: var(--arrow-left, 50%);
    z-index: -1;
  }

  .is-above .tooltip-arrow {
    bottom: -6px;
    transform: rotate(225deg);
  }

  .is-below .tooltip-arrow {
    top: -6px;
    transform: rotate(45deg);
  }

  .glass-effect {
    background: rgba(var(--color-card-rgb, 255, 255, 255), 0.8);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    border: 1px solid rgba(var(--color-border-rgb, 0, 0, 0), 0.1);
  }

  :global(.dark) .glass-effect {
    background: rgba(30, 30, 35, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
</style>
