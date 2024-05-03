import { writable } from "svelte/store";

// 画面
export enum Screen {
  Profile = 1,
  Wallpapers = 2,
  Extensions = 3,
}

export const screen = writable(Screen.Profile);
