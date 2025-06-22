import type { Application, WallpaperSource } from "./payload_config";

export type ApplyWallpaper = {
  application: Application | null;
  filters: string[] | null;
  opacity: number | null;
  source: WallpaperSource | null;
};
