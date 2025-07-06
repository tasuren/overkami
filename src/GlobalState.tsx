import {
  createContext,
  createEffect,
  createSignal,
  on,
  type ParentProps,
  useContext,
} from "solid-js";
import { getConfig } from "./lib/binding/command_config";
import type { Config, Wallpapers } from "./lib/binding/payload_config";
import type { View } from "./lib/view";

export interface GlobalState {
  wallpapers: () => Wallpapers;
  setWallpapers: (update: (prev: Wallpapers) => void) => void;
  view: () => View;
  setView: (view: View) => void;
  config: () => Config;
}

export const GlobalStateContext = createContext<GlobalState>();
const initialConfig = await getConfig();

export function GlobalStateProvider(props: ParentProps) {
  const [config, setConfig] = createSignal<Config>(initialConfig);
  const [wallpapers, setWallpapers] = createSignal<Wallpapers>(
    initialConfig.wallpapers,
    { equals: false },
  );
  const [view, setView] = createSignal<View>({ type: "home" });

  createEffect(
    on(wallpapers, (wallpapers) => {
      console.log(1);
      setConfig({
        version: config().version,
        wallpapers: wallpapers,
      });
    }),
  );

  const state: GlobalState = {
    wallpapers,
    setWallpapers,
    view,
    setView,
    config,
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

export function useConfig() {
  const state = useGlobalState();
  return [state.config] as const;
}
