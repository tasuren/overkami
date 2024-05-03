<script lang="ts">
  import { Screen, screen } from "$lib/state";

  let thisScreen: Screen;
  export { thisScreen as screen };

  $: isSelected = $screen == thisScreen;
</script>

<button
  class="rounded-md w-full px-2 py-1 block text-left cursor-pointer"
  class:dark:bg-secondary-dark={isSelected}
  class:bg-secondary-light={isSelected}
  class:no-background={!isSelected}
  on:click={() => ($screen = thisScreen)}
>
  <slot />
</button>

<style>
  /* リセットCSS */
  button {
    border: none;
    outline: none;
    font: inherit;
    color: inherit;
  }

  .no-background {
    background: none;
  }
</style>
