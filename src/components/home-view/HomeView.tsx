import { convertFileSrc } from "@tauri-apps/api/core";
import { basename } from "@tauri-apps/api/path";
import Plus from "lucide-solid/icons/plus";
import { For, Show, createEffect, createSignal } from "solid-js";
import { useView, useWallpapers } from "../../GlobalState";
import type { Wallpaper } from "../../lib/binding";
import { cl } from "../../lib/utils";
import { buttonClass, textMutedClass } from "../ui";

export function HomeView() {
  const [wallpapers] = useWallpapers();

  return (
    <>
      <Show when={wallpapers().length > 0} fallback={<NothingFound />}>
        <div
          class={cl(
            "px-14 py-10 h-full",
            "grid sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-3 xl:grid-cols-5",
            "gap-2 overflow-auto",
          )}
        >
          <For each={wallpapers()}>
            {(wallpaper, index) => (
              <WallpaperCard wallpaper={wallpaper} index={index()} />
            )}
          </For>
        </div>
      </Show>

      <AddButton />
    </>
  );
}

function AddButton() {
  const [, setView] = useView();
  const [wallpapers] = useWallpapers();

  const onClick = () => {
    setView({
      type: "wallpaper",
      wallpaper: undefined,
      index: wallpapers().length,
    });
  };

  return (
    <button
      type="button"
      class={buttonClass({
        class: "fixed bottom-14 left-14 p-4 h-fit rounded-xl",
      })}
      onClick={onClick}
    >
      <Plus class="stroke-dark dark:stroke-light" />
    </button>
  );
}

export function WallpaperCard(props: {
  wallpaper: Wallpaper;
  index: number;
}) {
  const { wallpaper, index } = props;
  const [, setView] = useView();

  const onClick = () => {
    setView({ type: "wallpaper", wallpaper, index });
  };

  const [displayAppName, setDisplayAppName] = createSignal(
    wallpaper.application.name,
  );

  createEffect(async () => {
    if (displayAppName() === null) {
      setDisplayAppName(await basename(wallpaper.application.path));
    }
  });

  return (
    <button
      type="button"
      onClick={onClick}
      class={cl("relative h-44 cursor-pointer", "active:scale-95 transition")}
    >
      <div class="h-44">
        <div class="w-full h-full">
          <Thumbnail wallpaper={wallpaper} />
        </div>
      </div>

      <div class="absolute bottom-0 left-0 backdrop-blur-lg w-full h-2/5 rounded-b-lg">
        <div class="h-full text-left flex flex-col justify-evenly px-3 py-2">
          <div class="font-mono text-xl">{wallpaper.name}</div>
          <div class={textMutedClass({ class: "font-mono" })}>
            {displayAppName()}
          </div>
        </div>
      </div>
    </button>
  );
}

function Thumbnail(props: { wallpaper: Wallpaper }) {
  const { wallpaper } = props;

  switch (wallpaper.source.type) {
    case "Picture":
      return (
        <img
          src={convertFileSrc(wallpaper.source.location)}
          alt="🖼"
          class="w-full h-full object-cover rounded-lg"
        />
      );
    case "Video":
      return (
        <video
          src={convertFileSrc(wallpaper.source.location)}
          class="w-full h-full object-cover rounded-lg"
          autoplay
          loop
          muted
        />
      );
    case "LocalWebPage":
      return (
        <iframe
          title={wallpaper.name}
          src={convertFileSrc(wallpaper.source.location)}
        />
      );
    case "RemoteWebPage":
      return <iframe title={wallpaper.name} src={wallpaper.source.location} />;
  }
}
function NothingFound() {
  return (
    <div class="h-full flex justify-center items-center">
      <div class={textMutedClass({ class: "text-center" })}>
        壁紙がまだ設定されていません。
        <br />
        ¯\_(ツ)_/¯
      </div>
    </div>
  );
}
