import { emit } from "@tauri-apps/api/event";
import type { AddWallpaper, ApplyWallpaper } from "./payload_wallpaper_event";

export async function applyWallpaper(payload: ApplyWallpaper) {
  await emit("apply-wallpaper", payload);
}

export async function addWallpaper(payload: AddWallpaper) {
  await emit("add-wallpaper", payload);
}

export async function removeWallpaper(id: string) {
  await emit("remove-wallpaper", id);
}
