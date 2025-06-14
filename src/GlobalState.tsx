import {
  type ParentProps,
  createContext,
  createSignal,
  useContext,
} from "solid-js";
import type { Wallpaper } from "./lib/binding";
import type { View } from "./lib/view";

export interface GlobalState {
  wallpapers: () => Wallpaper[];
  setWallpapers: (wallpapers: Wallpaper[]) => void;
  view: () => View;
  setView: (view: View) => void;
}

export const GlobalStateContext = createContext<GlobalState>();

export function GlobalStateProvider(props: ParentProps) {
  const [wallpapers, setWallpapers] = createSignal<Wallpaper[]>([]);
  const [view, setView] = createSignal<View>({ type: "home" });

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
