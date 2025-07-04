import { invoke } from "@tauri-apps/api/core";
import type { AddWallpaper, ApplyWallpaper } from "./payload_wallpaper_event";

export async function applyWallpaper(id: string, payload: ApplyWallpaper) {
  await invoke("apply-wallpaper", { id, payload });
}

export async function addWallpaper(id: string, payload: AddWallpaper) {
  await invoke("add-wallpaper", { id, payload });
}

export async function removeWallpaper(id: string) {
  await invoke("remove-wallpaper", { id });
}
