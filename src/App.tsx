import { Match, Switch } from "solid-js";
import "./App.css";
import { GlobalStateProvider, useView } from "./GlobalState";
import { HomeView } from "./components/home-view/HomeView";
import { default as WallpaperView } from "./components/wallpaper-view/WallpaperView";
import { cl } from "./lib/utils";

export function Main() {
  const [view] = useView();

  return (
    <main class="h-full pt-4">
      <Switch>
        <Match when={view().type === "home"}>
          <HomeView />
        </Match>
        <Match when={view()}>
          {(view) => {
            const wallpaperView = view();
            if (wallpaperView.type !== "wallpaper") return;
            console.log(wallpaperView.wallpaper);

            return <WallpaperView wallpaper={wallpaperView.wallpaper} />;
          }}
        </Match>
      </Switch>
    </main>
  );
}

function Header() {
  return (
    <div
      class="absolute top-0 left-0 w-screen h-12 z-50 flex justify-center items-center"
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
          "text-dark dark:text-light",
          "bg-light dark:bg-dark",
          "p-14",
        )}
      >
        <Header />
        <Main />
      </div>
    </GlobalStateProvider>
  );
}

export default App;
