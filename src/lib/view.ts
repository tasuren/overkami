import type { Wallpaper } from "./binding/payload_config";

export interface HomeView {
  type: "home";
}

export interface WallpaperView {
  type: "wallpaper";
  id: string;
  wallpaper: Wallpaper | undefined;
}

export type View = HomeView | WallpaperView;
