import { convertFileSrc } from "@tauri-apps/api/core";
import Plus from "lucide-solid/icons/plus";
import { For, Show } from "solid-js";
import { useView, useWallpapers } from "../../GlobalState";
import type { Wallpaper } from "../../lib/binding";
import { cl } from "../../lib/utils";
import { buttonClass, textMutedClass } from "../ui";

export function HomeView() {
  const [wallpapers] = useWallpapers();

  return (
    <>
      <Show when={wallpapers().length > 0} fallback={<NothingFound />}>
        <div class="px-14 py-10 grid sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-3 xl:grid-cols-5 gap-2 h-full">
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
        class: "fixed bottom-10 left-10 p-4 h-fit rounded-xl",
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

  return (
    <button
      type="button"
      class={cl(
        "h-44 p-4 rounded-xl cursor-pointer",
        "bg-card hover:bg-card/70 active:scale-95 transition",
        "flex flex-col",
      )}
      onClick={onClick}
    >
      <div class="p-2 grow">
        <Thumbnail wallpaper={wallpaper} />
      </div>
      <div class="space-y-1">
        <div class="xl">{wallpaper.name}</div>
        <div class="text-sm">{wallpaper.application.name}</div>
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
          class="object-cover"
        />
      );
    case "Video":
      return (
        <video
          src={convertFileSrc(wallpaper.source.location)}
          class="object-cover"
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
