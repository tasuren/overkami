import { emit } from "@tauri-apps/api/event";
import type { ApplyWallpaper } from "./payload_event";

export async function applyWallpaper(payload: ApplyWallpaper) {
  await emit("apply-wallpaper", payload);
}
