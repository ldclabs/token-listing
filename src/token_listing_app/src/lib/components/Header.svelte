<script lang="ts">
  import { fadeIn } from '$lib/animations/motion'
  import { getTheme, toggleTheme } from '$lib/stores/theme.svelte'

  let { children = undefined } = $props()

  const theme = $derived(getTheme())
</script>

<header
  use:fadeIn={{ y: -20, duration: 400 }}
  class="border-border-subtle sticky top-0 z-50 border-b shadow-sm backdrop-blur-xl"
>
  <div
    class="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-4 px-4 py-2 sm:px-8"
  >
    <div class="flex items-center gap-3">
      <a href="/" class="bg-white p-0.5"
        ><img
          alt="TokenList.ing Logo"
          src="/_assets/logo.webp"
          class="h-12 w-12 rounded-sm object-contain transition-transform hover:scale-105"
        /></a
      >
      <div>
        <p class="text-muted font-serif text-lg tracking-[0.25em]"
          >TokenList.ing</p
        >
        <p class="text-foreground hidden text-base font-medium md:flex">
          Token infrastructure for the multi-chain era
        </p>
      </div>
    </div>
    <div class="flex items-center gap-4 text-sm">
      {@render children?.()}
      <button
        class="border-border-subtle text-muted hover:border-foreground hover:text-foreground rounded-full border px-4 py-2 font-semibold tracking-wide uppercase transition-all hover:scale-105"
        onclick={toggleTheme}
        aria-pressed={theme === 'light'}
        aria-label="Toggle theme"
      >
        {theme === 'dark' ? '☀️' : '🌙'}
      </button>
    </div>
  </div>
</header>
