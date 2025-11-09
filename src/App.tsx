import { Show } from "solid-js";
import "./App.css";
import { HomeView } from "./components/home-view/HomeView";
import { default as WallpaperView } from "./components/wallpaper-view/WallpaperView";
import { GlobalStateProvider, useView } from "./GlobalState";
import { cl } from "./lib/utils";
import SofumeLogo from "./sofume_logo.png";

export function Main() {
  const [view] = useView();

  return (
    <main style="height: calc(100vh - 48px);">
      <img
        src={SofumeLogo}
        alt="千葉工業大学 ソフトメディア研究会"
        class="w-auto h-14 absolute -top-1 right-4 opacity-60"
      />

      <HomeView />

      <Show when={view().type === "wallpaper"}>
        {(_) => {
          const wallpaperView = view();
          if (wallpaperView.type !== "wallpaper") return;

          return (
            <div class="w-screen h-screen absolute top-0 left-0 bg-light dark:bg-dark">
              <WallpaperView
                id={wallpaperView.id}
                wallpaper={wallpaperView.wallpaper}
              />
            </div>
          );
        }}
      </Show>
    </main>
  );
}

function Header() {
  return (
    <div
      class={cl("w-screen h-12 z-50", "flex justify-center items-center")}
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
