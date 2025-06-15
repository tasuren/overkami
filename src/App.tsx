import { Match, Switch } from "solid-js";
import "./App.css";
import { GlobalStateProvider, useView } from "./GlobalState";
import { HomeView } from "./components/home-view/HomeView";
import { default as WallpaperView } from "./components/wallpaper-view/WallpaperView";
import { cl } from "./lib/utils";

export function Main() {
  const [view] = useView();

  return (
    <main style="height: calc(100vh - 48px);">
      <Switch>
        <Match when={view().type === "home"}>
          <HomeView />
        </Match>
        <Match when={view()}>
          {(view) => {
            const wallpaperView = view();
            if (wallpaperView.type !== "wallpaper") return;

            return (
              <WallpaperView
                wallpaper={wallpaperView.wallpaper}
                index={wallpaperView.index}
              />
            );
          }}
        </Match>
      </Switch>
    </main>
  );
}

function Header() {
  return (
    <div
      class={cl("w-screen h-[48px] z-50", "flex justify-center items-center")}
      data-tauri-drag-region
    >
      <h1 class="h-fit text-xl font-bold" data-tauri-drag-region>
        オーバーカミ！
      </h1>
    </div>
  );
}

function App() {
  return (
    <GlobalStateProvider>
      <div
        class={cl(
          "w-screen h-screen",
          "text-light-text dark:text-dark-text",
          "bg-light dark:bg-dark",
        )}
      >
        <Header />
        <Main />
      </div>
    </GlobalStateProvider>
  );
}

export default App;
