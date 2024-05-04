<script>
  import { onMount } from "svelte";

  import { getCurrent } from "@tauri-apps/api/window";
  import MenuItem from "./lib/components/ui/MenuItem.svelte";
  import TitleBar from "./lib/components/TitleBar.svelte";

  import { lazyThemeTransitionSetup } from "./lib";
  import { Screen, screen } from "$lib/state";
  import ProfileScreen from "$lib/ProfileScreen.svelte";
  import WallpapersScreen from "$lib/WallpapersScreen.svelte";
  import ExtensionsScreen from "$lib/ExtensionsScreen.svelte";

  onMount(() => lazyThemeTransitionSetup());

  getCurrent().startDragging();
</script>

<div class="flex h-full">
  <div class="w-1/4">
    <TitleBar />

    <div class="p-2 space-y-2">
      <MenuItem screen={Screen.Profile}>プロファイル</MenuItem>
      <MenuItem screen={Screen.Wallpapers}>壁紙</MenuItem>
      <MenuItem screen={Screen.Extensions}>拡張機能</MenuItem>
    </div>
  </div>

  <div
    class="
      w-3/4 outline outline-1 outline-white dark:outline-black
      bg-primary-light dark:bg-primary-dark
    "
  >
    <TitleBar class="outline outline-1 dark:outline-black box-border p-2">
      
    </TitleBar>

    <div class="p-4">
      {#if $screen == Screen.Profile}
        <ProfileScreen />
      {:else if $screen == Screen.Wallpapers}
        <WallpapersScreen />
      {:else if $screen == Screen.Extensions}
        <ExtensionsScreen />
      {/if}
    </div>
  </div>
</div>
