import { Show } from "solid-js";
import "./App.css";
import { GlobalStateProvider, useEditing } from "./GlobalState";
import { Home } from "./components/home-screen/HomeScreen";
import WallpaperScreen from "./components/wallpaper-screen/WallpaperScreen";
import { cl } from "./lib/utils";

export function Main() {
  const [editing] = useEditing();

  return (
    <main class="h-full pt-4">
      <Show when={editing() !== undefined} fallback={<Home />}>
        <WallpaperScreen wallpaper={editing()} />
      </Show>
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
