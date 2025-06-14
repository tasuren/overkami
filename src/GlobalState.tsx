import {
  type ParentProps,
  createContext,
  createSignal,
  useContext,
} from "solid-js";
import type { Wallpaper } from "./lib/binding";
import type { OptionalizeAll } from "./lib/utils";

export type EditingWallpaper = OptionalizeAll<Wallpaper>;

export interface GlobalState {
  wallpapers: () => Wallpaper[];
  setWallpapers: (wallpapers: Wallpaper[]) => void;
  editing: () => Wallpaper | EditingWallpaper | undefined;
  setEditing: (wallpaper: Wallpaper | EditingWallpaper | undefined) => void;
}

export const GlobalStateContext = createContext<GlobalState>();

export function GlobalStateProvider(props: ParentProps) {
  const [wallpapers, setWallpapers] = createSignal<Wallpaper[]>([]);
  const [editing, setEditing] = createSignal<Wallpaper>();

  const state: GlobalState = {
    wallpapers,
    setWallpapers,
    editing,
    setEditing,
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

export function useEditing() {
  const state = useGlobalState();
  return [state.editing, state.setEditing] as const;
}
