import { confirm } from "@tauri-apps/plugin-dialog";
import ChevronLeft from "lucide-solid/icons/chevron-left";
import { createSignal, Show } from "solid-js";
import { useView } from "../../GlobalState";
import type { Wallpaper } from "../../lib/binding/payload_config";
import { iconButtonClass, iconClass } from "../ui";
import WallpaperForm from "./WallpaperForm";

export default function WallpaperView(props: {
  id: string;
  wallpaper: Wallpaper | undefined;
}) {
  const { wallpaper, id } = props;
  const [, setView] = useView();
  const [dirty, setDirty] = createSignal(false);

  return (
    <>
      <div class="fixed top-12 left-0 px-14 z-50 w-screen flex items-center gap-2 bg-light dark:bg-dark">
        <button
          type="button"
          class={iconButtonClass()}
          onClick={async () => {
            if (dirty()) {
              if (
                !(await confirm(
                  "変更が保存されていません、それでも戻りますか？",
                ))
              ) {
                return;
              }
            }

            setView({ type: "home" });
          }}
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
        class="px-16 py-2 mt-24 overflow-y-auto"
        style="height: calc(100vh - 48px * 2);"
      >
        <WallpaperForm id={id} wallpaper={wallpaper} setDirty={setDirty} />
      </div>
    </>
  );
}
