export type RemoteWebPageSource = {
  type: "RemoteWebPage";
  location: string;
};

export type LocalWebPageSource = {
  type: "LocalWebPage";
  location: string;
};

export type YouTubeSource = {
  type: "YouTube";
  location: string;
};

export type PictureSource = {
  type: "Picture";
  location: string;
};

export type VideoSource = {
  type: "Video";
  location: string;
};

export type RemoteWallpaperSource = RemoteWebPageSource | YouTubeSource;
export type LocalWallpaperSource =
  | LocalWebPageSource
  | PictureSource
  | VideoSource;
export type WallpaperSource = RemoteWallpaperSource | LocalWallpaperSource;

export const STRING_FILTER_STRATEGIES = {
  Prefix: "前方一致",
  Suffix: "後方一致",
  Contains: "部分一致",
  Exact: "完全一致",
} as const;
export type StringFilterStrategy = keyof typeof STRING_FILTER_STRATEGIES;

export type WindowNameFilter = {
  type: "WindowName";
  name: string;
  strategy: StringFilterStrategy;
};

export type Filter = WindowNameFilter;

export type Wallpaper = {
  name: string;
  applicationName: string;
  filters: Filter[];
  source: WallpaperSource;
  opacity: number;
};

export type Wallpapers = { [key: string]: Wallpaper };

export type Config = {
  version: string;
  open_window_on_startup: boolean,
  wallpapers: Wallpapers;
};
