import type { Filter, Wallpaper, WallpaperSource } from "./payload_config";

export type ApplyWallpaper = {
  id: string;
  name?: string;
  applicationPath?: string;
  filters?: Filter[];
  opacity?: number;
  source?: WallpaperSource;
};

export type AddWallpaper = {
  id: string;
  wallpaper: Wallpaper;
};
