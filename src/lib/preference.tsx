import {
    Component,
    createContext,
    createEffect,
    createMemo,
    createSignal,
    ParentProps
} from "solid-js";
import { Store } from "@tauri-apps/plugin-store";
import { join } from "@tauri-apps/api/path";
import { paths } from "./data";

// Types
export const THEMES = ["dark", "light", "system"] as const;
export type Theme = (typeof THEMES)[number];

// Store
const store = new Store(await join(paths.app_config_dir, "state.bin"));

async function readTheme(): Promise<Theme> {
    return (await store.get<Theme>("theme")) || "system";
}

async function saveTheme(theme: Theme): Promise<void> {
    await store.set("theme", theme);
}

// Theme setup
export const defaultTheme = await readTheme();

/** Apply theme to document. */
function applyTheme(theme: Theme) {
    const absoluteTheme =
        theme == "system"
            ? window.matchMedia("(prefers-color-scheme: dark)").matches
                ? "dark"
                : "light"
            : theme;
    document.documentElement.style.colorScheme = absoluteTheme;
    document.documentElement.setAttribute("data-theme", absoluteTheme);

    saveTheme(theme);
}

applyTheme(defaultTheme);

export const PreferenceContext = createContext({
    theme() {
        return defaultTheme;
    },
    setTheme(theme: Theme) {
        theme;
    }
});

export const PreferenceProvider: Component<ParentProps> = (props) => {
    const [theme, setTheme] = createSignal(defaultTheme);
    const themeMemo = createMemo(theme);
    const state = {
        theme: themeMemo,
        setTheme
    };

    console.log(2);
    createEffect(() => {
        console.log(1);
        applyTheme(theme());
    });

    return (
        <>
            <PreferenceContext.Provider value={state}>
                {props.children}
            </PreferenceContext.Provider>
        </>
    );
};
