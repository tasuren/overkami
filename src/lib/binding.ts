import { invoke } from "@tauri-apps/api/core";
import { message } from "@tauri-apps/plugin-dialog";

async function errorMessage(error: Error) {
  if (typeof error === "string") {
    await message(`不明なエラーが発生しました。\n詳細: ${error}`, {
      kind: "error",
    });
  } else {
    await message(`${error.message}\n詳細: ${error.detail}`, { kind: "error" });
  }
}

export type Error =
  | string
  | {
      message: string;
      detail: string;
    };

export type Application = {
  name: string | null;
  path: string;
};

export type RemoteWebPageSource = {
  type: "RemoteWebPage";
  location: string;
};

export type LocalWebPageSource = {
  type: "LocalWebPage";
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

export type RemoteWallpaperSource = RemoteWebPageSource;
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
  application: Application;
  filters: Filter[];
  source: WallpaperSource;
  opacity: number;
};

export type Config = {
  version: string;
  wallpapers: Wallpaper[];
};

export async function getConfig(): Promise<Config> {
  return await invoke("get_config");
}

export async function saveConfig(config: Config): Promise<void> {
  try {
    return await invoke("save_config", { config });
  } catch (error) {
    errorMessage(error as Error);
  }
}

export interface ApplicationWindow {
  windowTitle: string | null;
  name: string | null;
  path: string;
}

export async function getApplicationWindows(): Promise<ApplicationWindow[]> {
  return await invoke("get_application_windows");
}
