import {
  type ParentProps,
  createContext,
  createEffect,
  createSignal,
  useContext,
} from "solid-js";
import {
  type Config,
  type Wallpaper,
  getConfig,
  saveConfig,
} from "./lib/binding";
import type { View } from "./lib/view";

export interface GlobalState {
  wallpapers: () => Wallpaper[];
  setWallpapers: (update: (prev: Wallpaper[]) => void) => void;
  view: () => View;
  setView: (view: View) => void;
}

export const GlobalStateContext = createContext<GlobalState>();
const initialConfig = await getConfig();

export function GlobalStateProvider(props: ParentProps) {
  let config: Config = initialConfig;
  const [wallpapers, setWallpapers] = createSignal<Wallpaper[]>([], {
    equals: false,
  });
  const [view, setView] = createSignal<View>({ type: "home" });

  createEffect(async () => {
    config = await getConfig();
    setWallpapers(config.wallpapers);
  });

  createEffect(() => {
    config.wallpapers = wallpapers();

    saveConfig(config);
  });

  const state: GlobalState = {
    wallpapers,
    setWallpapers,
    view,
    setView,
  };

  return (
    <GlobalStateContext.Provider value={state}>
      {props.children}
    </GlobalStateContext.Provider>
  );
}

export function useGlobalState() {
  const state = useContext(GlobalStateContext);
  if (!state)
    throw new Error(
      "`useGlobalState` must be used within a `GlobalStateProvider`.",
    );

  return state;
}

export function useWallpapers() {
  const state = useGlobalState();
  return [state.wallpapers, state.setWallpapers] as const;
}

export function useView() {
  const state = useGlobalState();
  return [state.view, state.setView] as const;
}
