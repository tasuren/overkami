import ChevronLeft from "lucide-solid/icons/chevron-left";
import { Show, createSignal } from "solid-js";
import { type EditingWallpaper, useEditing } from "../../GlobalState";
import { iconButtonClass, iconClass } from "../ui";
import WallpaperForm from "./WallpaperForm";

export default function WallpaperScreen(props: {
  wallpaper?: EditingWallpaper;
}) {
  const { wallpaper } = props;
  const [name, setName] = createSignal(undefined);
  const [, setEditing] = useEditing();

  return (
    <div>
      <div class="flex items-center gap-2 mb-4">
        <button
          type="button"
          class={iconButtonClass()}
          onClick={() => setEditing(undefined)}
        >
          <ChevronLeft
            class={iconClass({ class: "cursor-pointer" })}
            size={30}
          />
        </button>
        <h1 class="text-2xl">
          <Show when={name() === undefined} fallback={`壁紙: ${name()}`}>
            新しい壁紙
          </Show>
        </h1>
      </div>

      <WallpaperForm wallpaper={wallpaper} />
    </div>
  );
}
