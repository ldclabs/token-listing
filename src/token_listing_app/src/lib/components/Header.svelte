<script lang="ts">
  import { fadeIn } from '$lib/animations/motion'
  import MoonLine from '$lib/icons/moon-line.svelte'
  import SunLine from '$lib/icons/sun-line.svelte'
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
  class="border-border/40 bg-background/60 sticky top-0 z-50 border-b backdrop-blur-md"
>
  <div
    class="mx-auto flex max-w-6xl items-center justify-between gap-4 px-4 py-3 sm:px-8"
  >
    <div class="flex items-center gap-4">
      <a
        href={backUrl}
        class="group flex items-center gap-3 transition-opacity hover:opacity-80"
      >
        <img
          alt="TokenList.ing Logo"
          src="/_assets/logo.webp"
          class="h-10 w-10 rounded-xl object-contain transition-transform duration-500 group-hover:scale-105 dark:invert"
        />
        <div class="hidden flex-col sm:flex">
          <span
            class="text-foreground font-serif text-xl font-bold tracking-tight"
          >
            TokenList.ing
          </span>
          <span
            class="text-muted-foreground text-[10px] font-medium tracking-widest uppercase"
          >
            {description}
          </span>
        </div>
      </a>
    </div>

    <div class="flex items-center gap-2 sm:gap-6">
      <div class="hidden items-center md:flex">
        {@render children?.()}
      </div>

      <div
        class="border-border/40 flex items-center gap-2 border-l pl-2 sm:gap-3 sm:pl-6"
      >
        {#if authStore.identity.isAuthenticated}
          <button
            class="glass-border bg-surface/50 hover:bg-surface text-muted hover:text-foreground flex h-10 w-10 items-center justify-center rounded-xl transition-all"
            onclick={onUserProfileModal}
            title="User Profile"
          >
            <User3Line class="size-5" />
          </button>
        {:else}
          <Button
            class="bg-foreground text-background flex h-10 items-center gap-2 rounded-xl px-4 text-sm font-bold transition-all hover:opacity-90 disabled:opacity-50"
            onclick={onSignWith}
            isLoading={isSigningIn}
          >
            <User3Line class="size-5" />
            <span class="hidden sm:inline">Sign in</span>
          </Button>
        {/if}

        <button
          class="glass-border bg-surface/50 hover:bg-surface text-muted hover:text-foreground flex h-10 w-10 items-center justify-center rounded-xl transition-all"
          onclick={toggleTheme}
          aria-pressed={theme === 'light'}
          aria-label="Toggle theme"
        >
          {#if theme === 'dark'}
            <SunLine class="size-5" />
          {:else}
            <MoonLine class="size-5" />
          {/if}
        </button>
      </div>
    </div>
  </div>
</header>
