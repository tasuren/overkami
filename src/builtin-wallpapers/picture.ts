import { invoke } from "@tauri-apps/api/core";

addEventListener("load", async () => {
  await invoke("get_wallpaper_config", {});
});
