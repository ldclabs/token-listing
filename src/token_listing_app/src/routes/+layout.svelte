<script lang="ts">
  import { initTheme } from '$lib/stores/theme.svelte'
  import Toast from '$lib/ui/Toast.svelte'
  import { onMount } from 'svelte'
  import '../app.css'

  let { children } = $props()

  // type Theme = 'light' | 'dark'
  // let theme = $state<Theme>('dark')
  // const setTheme = (value: Theme) => {
  //   theme = value
  //   localStorage['theme'] = value

  //   document.documentElement.classList.toggle(
  //     'dark',
  //     localStorage['theme'] === 'dark' ||
  //       (!('theme' in localStorage) &&
  //         window.matchMedia('(prefers-color-scheme: dark)').matches)
  //   )
  // }
  // const themeValue = $derived(() => theme)
  // setContext('theme', [
  //   () => themeValue,
  //   () => setTheme(theme === 'dark' ? 'light' : 'dark')
  // ])
  onMount(() => {
    initTheme()
  })
</script>

{@render children()}

<Toast />
