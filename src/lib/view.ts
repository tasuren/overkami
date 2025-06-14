import type { Wallpaper } from "./binding";

export interface HomeView {
  type: "home";
}

export interface WallpaperView {
  type: "wallpaper";
  wallpaper: Wallpaper | undefined;
}

export type View = HomeView | WallpaperView;
