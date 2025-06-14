import { invoke } from "@tauri-apps/api/core";

export interface Application {
  name?: string;
  path: string;
}

export interface Wallpaper {
  name: string;
  application: Application;
}

export interface ApplicationWindow {
  windowTitle?: string;
  name?: string;
  path: string;
}

export async function getApplicationWindows(): Promise<ApplicationWindow[]> {
  return await invoke("get_application_windows");
}
