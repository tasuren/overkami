import ChevronLeft from "lucide-solid/icons/chevron-left";
import { Show } from "solid-js";
import { useView } from "../../GlobalState";
import type { Wallpaper } from "../../lib/binding";
import { iconButtonClass, iconClass } from "../ui";
import WallpaperForm from "./WallpaperForm";

export default function WallpaperView(props: {
  wallpaper: Wallpaper | undefined;
}) {
  const { wallpaper } = props;
  const [, setView] = useView();

  return (
    <div>
      <div class="fixed top-[48px] left-0 px-14 z-50 w-screen flex items-center gap-2 bg-light dark:bg-dark">
        <button
          type="button"
          class={iconButtonClass()}
          onClick={() => setView({ type: "home" })}
        >
          <ChevronLeft
            class={iconClass({ class: "cursor-pointer" })}
            size={30}
          />
        </button>
        <h1 class="text-2xl">
          <Show when={wallpaper?.name} fallback="新しい壁紙">
            {`壁紙: ${wallpaper?.name}`}
          </Show>
        </h1>
      </div>

      <div
        class="px-16 py-8 mt-[48px] overflow-y-auto"
        style="height: calc(100vh - 48px * 2);"
      >
        <WallpaperForm wallpaper={wallpaper} />
      </div>
    </div>
  );
}
