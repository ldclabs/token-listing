<script lang="ts">
  import type { Snippet } from 'svelte'
  import { tick } from 'svelte'
  import { quintOut } from 'svelte/easing'
  import { slide } from 'svelte/transition'

  let {
    open = $bindable(false),
    disabled = false,
    trigger,
    children,
    containerClass = '',
    triggerClass = '',
    menuClass = ''
  }: {
    open?: boolean
    disabled?: boolean
    trigger: (props: { open: boolean }) => any
    children: Snippet<[]>
    containerClass?: string
    triggerClass?: string
    menuClass?: string
  } = $props()

  let containerEl = $state<HTMLElement | null>(null)
  let triggerWrapperEl = $state<HTMLElement | null>(null)
  let menuEl = $state<HTMLElement | null>(null)
  let menuStyle = $state('opacity: 0;')

  function close() {
    open = false
  }

  function toggle() {
    open = !open
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      close()
    }
  }

  function portal(node: HTMLElement) {
    document.body.appendChild(node)
    return () => {
      if (node.parentNode === document.body) document.body.removeChild(node)
    }
  }

  $effect(() => {
    if (!open) {
      menuStyle = 'opacity: 0;'
      return
    }

    if (open && triggerWrapperEl) {
      tick().then(() => {
        if (!menuEl || !triggerWrapperEl) return

        const triggerRect = triggerWrapperEl.getBoundingClientRect()
        const menuHeight = menuEl.offsetHeight
        const spaceBelow = window.innerHeight - triggerRect.bottom
        const spaceAbove = triggerRect.top

        let top = 0
        let left = Math.max(
          16,
          triggerRect.left - (menuEl.offsetWidth - triggerRect.width) / 2
        )
        left = Math.min(left, window.innerWidth - menuEl.offsetWidth - 16)

        if (spaceBelow < menuHeight && spaceAbove > menuHeight) {
          top = Math.max(triggerRect.top - menuHeight - 8, 16)
        } else {
          top = triggerRect.bottom + 8
        }

        menuStyle = `left: ${left}px; top: ${top}px; opacity: 1;`
      })
    }

    if (!open || !containerEl) return

    const handleClickOutside = (event: MouseEvent) => {
      if (!containerEl?.contains(event.target as Node)) {
        close()
      }
    }

    const timeoutId = setTimeout(() => {
      document.addEventListener('click', handleClickOutside, true)
    }, 0)

    return () => {
      clearTimeout(timeoutId)
      document.removeEventListener('click', handleClickOutside, true)
    }
  })
</script>

<div
  class={'dropdown-container ' + containerClass}
  bind:this={containerEl}
  onkeydown={handleKeydown}
  role="presentation"
>
  <button
    class={'dropdown-trigger ' + triggerClass}
    bind:this={triggerWrapperEl}
    {disabled}
    onclick={toggle}
    aria-haspopup="true"
    aria-expanded={open}
  >
    {@render trigger({ open })}
  </button>

  {#if open}
    <div
      {@attach portal}
      class={'dropdown-menu ' + menuClass}
      bind:this={menuEl}
      style={menuStyle}
      role="menu"
      transition:slide={{ duration: 200, easing: quintOut }}
    >
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .dropdown-container {
    position: relative;
  }

  .dropdown-trigger {
    cursor: pointer;
  }

  .dropdown-menu {
    position: fixed;
    z-index: 999;
    overflow: auto; /* 当菜单内容过多时，内部可以滚动 */
    max-height: 40vh; /* 限制最大高度，防止超出屏幕 */
    transition:
      opacity 0.15s ease-in-out,
      transform 0.15s ease-in-out;
    transform: scaleY(0.95) translateY(-10px);
    transform-origin: top;
  }

  .dropdown-menu[style*='opacity: 1'] {
    transform: scaleY(1) translateY(0);
  }

  /* 7. 向上翻转时的效果 */
  .dropdown-menu.flipped {
    transform-origin: bottom;
    transform: scaleY(0.95) translateY(10px);
  }

  .dropdown-menu.flipped[style*='opacity: 1'] {
    transform: scaleY(1) translateY(0);
  }
</style>
