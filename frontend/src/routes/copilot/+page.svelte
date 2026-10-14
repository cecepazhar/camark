<script lang="ts">
  import { goto } from '$app/navigation';
  import { getAiChatState, toggleAiChat } from '$lib/stores/aiChat.svelte';
  import { onMount } from 'svelte';

  const aiChat = getAiChatState();

  onMount(() => {
    if (!aiChat.open) {
      toggleAiChat();
    }
    goto('/', { replaceState: true });
  });
</script>
