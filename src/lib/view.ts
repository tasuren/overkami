import type { Wallpaper } from "./binding/payload_config";

export interface HomeView {
  type: "home";
}

export interface WallpaperView {
  type: "wallpaper";
  wallpaper: Wallpaper | undefined;
  index: number;
}

export type View = HomeView | WallpaperView;
