import Plus from "lucide-solid/icons/plus";
import { For } from "solid-js";
import { useEditing, useWallpapers } from "../../GlobalState";
import type { Wallpaper } from "../../lib/binding";
import { cl } from "../../lib/utils";
import { buttonClass } from "../ui";

export function Home() {
  const [wallpapers] = useWallpapers();

  return (
    <>
      <div class="grid sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-3 xl:grid-cols-5 gap-2 h-full">
        <For each={wallpapers()}>
          {(wallpaper) => <WallpaperCard wallpaper={wallpaper} />}
        </For>
      </div>

      <AddButton />
    </>
  );
}

function AddButton() {
  const [_, setEditing] = useEditing();

  const onClick = () => {
    setEditing({});
  };

  return (
    <button
      type="button"
      class={buttonClass({
        class: "absolute bottom-10 left-10 p-4 h-fit rounded-xl",
      })}
      onClick={onClick}
    >
      <Plus class="stroke-dark dark:stroke-light" />
    </button>
  );
}

export function WallpaperCard(props: {
  wallpaper: Wallpaper;
}) {
  const { wallpaper } = props;
  const [_, setEditing] = useEditing();

  const onClick = () => {
    setEditing(wallpaper);
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
      <div class="p-2 grow">あいうえお</div>
      <div class="space-y-1">
        <div class="xl">{wallpaper.name}</div>
        <div class="text-sm">{wallpaper.application.name}</div>
      </div>
    </button>
  );
}
