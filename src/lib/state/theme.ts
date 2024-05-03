import { get, writable } from "svelte/store";

export const STATIC_THEMES = {
  dark: "ダークモード",
  light: "ライトモード"
} as const;
export const DYNAMIC_THEMES = { system: "自動" };

export type StaticTheme = keyof typeof STATIC_THEMES;
export type DynamicTheme = keyof typeof DYNAMIC_THEMES;
export type Theme = StaticTheme | DynamicTheme;

export const THEMES = Object.assign(STATIC_THEMES, DYNAMIC_THEMES);
export const themeOrder: Theme[] = ["dark", "light", "system"];
const THEME_MEDIA = "(prefers-color-scheme: dark)";

function getSystemTheme(): StaticTheme {
  return window.matchMedia(THEME_MEDIA).matches ? "dark" : "light";
}

// テーマを取得し、なければOS設定から汲み取る。
export const theme = writable((localStorage.getItem("theme") as Theme) || "light");
export const staticTheme = writable("dark");

window.matchMedia(THEME_MEDIA).addEventListener("change", (event) => {
  const currentTheme = get(theme);
  if (currentTheme === "system") theme.set(event.matches ? "dark" : "light");
});

theme.subscribe((theme) => {
  localStorage.setItem("theme", theme);
  staticTheme.set(theme === "system" ? getSystemTheme() : theme);

  if (get(staticTheme) == "dark") document.documentElement.setAttribute("data-theme", "dark");
  else document.documentElement.removeAttribute("data-theme");

  document.documentElement.style.colorScheme = get(staticTheme);
})

/* API*/
export function next(target: Theme): Theme {
  for (const i in Object.entries(themeOrder)) {
    if (themeOrder[i] == target) {
      const tmpTheme = themeOrder[+i + 1];
      theme.set(tmpTheme ? (target = tmpTheme) : themeOrder[0]);
      return get(theme);
    }
  }

  throw new Error("unreacheable");
}
export function isDark(): boolean {
  return get(theme) == "dark";
}
