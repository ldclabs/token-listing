<script lang="ts">
  import { fadeIn } from '$lib/animations/motion'
  import User3Line from '$lib/icons/user-3-line.svelte'
  import { authStore } from '$lib/stores/auth.svelte'
  import { showModal } from '$lib/stores/modal.svelte'
  import { getTheme, toggleTheme } from '$lib/stores/theme.svelte'
  import { toastRun } from '$lib/stores/toast.svelte'
  import Button from '$lib/ui/Button.svelte'
  import { tick } from 'svelte'
  import UserProfileModal from './UserProfileModal.svelte'

  let {
    description = 'Token infrastructure for the multi-chain era',
    backUrl = '/',
    children = undefined
  } = $props()

  const theme = $derived(getTheme())

  let isSigningIn = $state(false)
  function onSignWith() {
    if (isSigningIn) return

    isSigningIn = true
    const result = authStore.signIn()

    toastRun(async () => {
      await result
      await tick()
      // Additional actions after sign-in can be placed here
    }).finally(() => {
      isSigningIn = false
    })
  }

  function onUserProfileModal() {
    showModal({
      title: 'User Profile',
      component: UserProfileModal,
      size: 'md'
    })
  }
</script>

<header
  use:fadeIn={{ y: -20, duration: 400 }}
  class="border-border-subtle sticky top-0 z-50 border-b shadow-sm backdrop-blur-xl"
>
  <div
    class="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-4 px-4 py-2 sm:px-8"
  >
    <div class="flex items-center gap-3">
      <a href={backUrl} class="bg-white p-0.5"
        ><img
          alt="TokenList.ing Logo"
          src="/_assets/logo.webp"
          class="h-12 w-12 rounded-sm object-contain transition-transform hover:scale-105"
        /></a
      >
      <div>
        <p class="text-muted font-serif text-lg tracking-widest"
          >TokenList.ing</p
        >
        <p class="text-foreground hidden text-base font-medium md:flex">
          {description}
        </p>
      </div>
    </div>
    <div class="flex items-center gap-4 text-sm">
      {@render children?.()}
      {#if authStore.identity.isAuthenticated}
        <button
          class=" text-muted hover:border-foreground hover:text-foreground inline-flex items-center gap-2 rounded-full px-4 py-2 transition-all"
          onclick={onUserProfileModal}
        >
          <User3Line class="size-5" />
        </button>
      {:else}
        <Button
          class="bg-foreground text-background inline-flex items-center gap-2 rounded-full px-4 py-2 transition-all hover:opacity-90 disabled:opacity-50"
          onclick={onSignWith}
          isLoading={isSigningIn}
        >
          <User3Line class="size-5" />
          Sign in
        </Button>
      {/if}
      <button
        class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-4 py-2 transition-all hover:scale-105"
        onclick={toggleTheme}
        aria-pressed={theme === 'light'}
        aria-label="Toggle theme"
      >
        {theme === 'dark' ? '☀️' : '🌙'}
      </button>
    </div>
  </div>
</header>
