<script lang="ts">
  import Toast from '$lib/ui/Toast.svelte'
  import '../app.css'

  let { children } = $props()
</script>

{@render children()}

<Toast />
