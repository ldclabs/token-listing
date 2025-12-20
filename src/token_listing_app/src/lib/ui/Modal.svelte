<script lang="ts">
  import { portal } from '$lib/actions/portal'
  import CloseLine from '$lib/icons/close-line.svelte'
  import {
    modalStore,
    type ModalAction,
    type ModalInstance
  } from '$lib/stores/modal.svelte.ts'
  import type { Action } from 'svelte/action'
  import { cubicOut } from 'svelte/easing'
  import { fade, scale } from 'svelte/transition'

  const sizeClasses = {
    sm: 'sm:max-w-sm',
    md: 'sm:max-w-lg',
    lg: 'sm:max-w-2xl',
    xl: 'sm:max-w-4xl',
    full: 'h-full sm:max-w-5xl lg:max-w-6xl xl:max-w-7xl'
  } as const

  const actionVariants = {
    primary:
      'inline-flex h-10 items-center justify-center rounded-full bg-sky-500 px-5 text-sm font-semibold text-white shadow-sm transition hover:bg-sky-400 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-400 disabled:cursor-not-allowed disabled:bg-sky-300',
    secondary:
      'inline-flex h-10 items-center justify-center rounded-full border border-slate-200 bg-white px-5 text-sm font-semibold text-slate-700 shadow-sm transition hover:border-slate-300 hover:bg-slate-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-300 disabled:cursor-not-allowed disabled:opacity-60 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100 dark:hover:bg-slate-800',
    ghost:
      'inline-flex h-10 items-center justify-center rounded-full px-5 text-sm font-semibold text-slate-600 transition hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-300 disabled:cursor-not-allowed disabled:opacity-60 dark:text-slate-200 dark:hover:bg-slate-800',
    danger:
      'inline-flex h-10 items-center justify-center rounded-full bg-rose-500 px-5 text-sm font-semibold text-white shadow-sm transition hover:bg-rose-400 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-rose-400 disabled:cursor-not-allowed disabled:bg-rose-300'
  } as const

  const focusableSelector =
    'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'

  const lifecycle: Action<HTMLDivElement, ModalInstance> = (node, modal) => {
    const previousFocus = document.activeElement as HTMLElement | null

    const getFocusable = () =>
      Array.from(node.querySelectorAll<HTMLElement>(focusableSelector)).filter(
        (element) =>
          !element.hasAttribute('disabled') &&
          element.getAttribute('aria-hidden') !== 'true'
      )

    const focusInitial = () => {
      const autofocus = node.querySelector<HTMLElement>('[data-autofocus]')
      const fallback = node.querySelector<HTMLElement>('[autofocus]')
      const target = autofocus ?? fallback ?? getFocusable()[0] ?? node
      target.focus({ preventScroll: true })
    }

    const trapTab = (event: KeyboardEvent) => {
      if (event.key !== 'Tab') return
      const focusable = getFocusable()
      if (!focusable.length) {
        event.preventDefault()
        node.focus({ preventScroll: true })
        return
      }

      const current = document.activeElement as HTMLElement
      const index = focusable.indexOf(current)
      const normalizedIndex = index >= 0 ? index : 0
      const direction = event.shiftKey ? -1 : 1
      const nextIndex =
        (normalizedIndex + direction + focusable.length) % focusable.length
      const target = focusable[nextIndex] ?? focusable[0] ?? node
      event.preventDefault()
      target.focus({ preventScroll: true })
    }

    const handleKeydown = (event: KeyboardEvent) => {
      if (modal.id !== topModalId) return
      if (event.key === 'Escape' && modal.closeOnEscape) {
        event.preventDefault()
        modal.close('escape')
        return
      }
      if (event.key === 'Tab') {
        trapTab(event)
      }
    }

    const handleFocusIn = (event: FocusEvent) => {
      if (modal.id !== topModalId) return
      if (node.contains(event.target as Node)) return
      focusInitial()
    }

    const raf = requestAnimationFrame(focusInitial)
    document.addEventListener('keydown', handleKeydown, true)
    document.addEventListener('focusin', handleFocusIn)

    return {
      destroy() {
        cancelAnimationFrame(raf)
        document.removeEventListener('keydown', handleKeydown, true)
        document.removeEventListener('focusin', handleFocusIn)
        previousFocus?.focus({ preventScroll: true })
      }
    }
  }

  function init(node: HTMLDivElement) {
    const checkState = () => {
      if (node.childElementCount > 0) {
        try {
          node.hidePopover()
        } catch {}
        try {
          node.showPopover()
        } catch {}
      } else {
        try {
          node.hidePopover()
        } catch {}
      }
    }

    checkState()

    const observer = new MutationObserver(checkState)
    observer.observe(node, { childList: true })

    return () => observer.disconnect()
  }

  const topModalId = $derived(modalStore.at(-1)?.id || 0)

  function handleBackdrop(modal: ModalInstance) {
    if (modal.id !== topModalId) return
    if (!modal.closeOnBackdrop) return
    modal.close('backdrop')
  }

  function handleClose(modal: ModalInstance) {
    if (!modal.dismissable) return
    modal.close('close-button')
  }

  async function handleAction(modal: ModalInstance, action: ModalAction) {
    if (action.disabled) return

    try {
      const result = action.handler?.((reason) => {
        modal.close(reason ?? 'action')
      })
      if (result instanceof Promise) {
        const resolved = await result
        if (resolved === false) return
      } else if (result === false) {
        return
      }
    } catch (error) {
      console.error(error)
      return
    }

    if (action.closeOnClick ?? true) {
      modal.close('action')
    }
  }
</script>

<div
  use:portal
  {@attach init}
  popover="manual"
  class="pointer-events-auto fixed inset-0 z-9998 flex h-full w-full flex-col items-center justify-center bg-transparent"
  class:pointer-events-none={!modalStore.length}
>
  {#each modalStore as modal, index (modal.id)}
    <div
      class="pointer-events-auto relative flex h-full w-full items-start justify-center p-0 pt-8 sm:p-8 lg:pt-16"
      style={`z-index: ${index + 1}`}
    >
      <div
        class="absolute inset-0 z-0 h-full w-full bg-slate-900/10 backdrop-blur-[1px] dark:bg-slate-950/70"
        transition:fade={{ duration: 150 }}
        aria-hidden="true"
        onclick={() => handleBackdrop(modal)}
      ></div>
      <div
        class={`relative z-10 flex max-h-full w-full flex-col gap-6 overflow-y-auto bg-white/98 p-4 text-slate-900 shadow-xl ring-1 ring-black/5 focus:outline-none sm:rounded-xl sm:p-6 dark:bg-slate-900/95 dark:text-slate-100  ${sizeClasses[modal.size ?? 'md']}`}
        role="dialog"
        aria-modal="true"
        aria-labelledby={modal.title ? `modal-title-${modal.id}` : undefined}
        aria-describedby={modal.description
          ? `modal-description-${modal.id}`
          : undefined}
        tabindex="-1"
        use:lifecycle={modal}
        transition:scale={{ duration: 180, easing: cubicOut }}
      >
        {#if modal.dismissable}
          <button
            type="button"
            class="absolute top-5 right-5 inline-flex size-9 items-center justify-center rounded-full border border-transparent text-slate-400 transition hover:bg-slate-100 hover:text-slate-600 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-200 dark:text-slate-400 dark:hover:bg-slate-800 dark:hover:text-slate-200"
            aria-label="Close dialog"
            onclick={() => handleClose(modal)}
          >
            <CloseLine />
          </button>
        {/if}

        <div class="flex flex-col gap-3 pr-10">
          {#if modal.title}
            <h2
              id={`modal-title-${modal.id}`}
              class="text-xl leading-7 font-semibold text-slate-900 dark:text-slate-100"
            >
              {modal.title}
            </h2>
          {/if}
          {#if modal.description}
            <p
              id={`modal-description-${modal.id}`}
              class="text-sm leading-6 text-slate-600 dark:text-slate-300"
            >
              {modal.description}
            </p>
          {/if}
          {#if modal.message}
            <p class="text-base leading-6 text-slate-700 dark:text-slate-200">
              {modal.message}
            </p>
          {/if}
        </div>

        {#if modal.content}
          <div class="text-sm leading-6 text-slate-700 dark:text-slate-200">
            {@render modal.content()}
          </div>
        {:else if modal.component}
          {@const DynamicComponent = modal.component}
          <DynamicComponent
            closeModal={() => modal.close()}
            {...modal.componentProps ?? {}}
          />
        {/if}

        {#if modal.actions.length}
          <div class="mt-2 flex flex-wrap items-center justify-end gap-3">
            {#each modal.actions as action (action.id ?? action.label)}
              <button
                type="button"
                class={actionVariants[action.variant ?? 'secondary']}
                disabled={action.disabled}
                data-autofocus={action.autofocus ? 'true' : undefined}
                onclick={() => handleAction(modal, action)}
              >
                {action.label}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/each}
</div>
