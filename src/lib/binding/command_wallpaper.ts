import { invoke } from "@tauri-apps/api/core";
import type { AddWallpaper, ApplyWallpaper } from "./payload_wallpaper";

export async function applyWallpaper(id: string, payload: ApplyWallpaper) {
  await invoke("apply_wallpaper", { id, payload });
}

export async function addWallpaper(id: string, payload: AddWallpaper) {
  await invoke("add_wallpaper", { id, payload });
}

export async function removeWallpaper(id: string) {
  await invoke("remove_wallpaper", { id });
}
