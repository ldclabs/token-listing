<script lang="ts">
  import { initTheme } from '$lib/stores/theme.svelte'
  import Modal from '$lib/ui/Modal.svelte'
  import Toast from '$lib/ui/Toast.svelte'
  import { AuthStore } from '$src/lib/stores/auth.svelte'
  import { onMount } from 'svelte'
  import '../app.css'

  let { children } = $props()

  onMount(() => {
    initTheme()
    AuthStore.init().catch((err) => {
      console.error(err)
    })
  })
</script>

{@render children()}

<Toast />
<Modal />
