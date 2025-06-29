import {
  Form,
  type SubmitHandler,
  createFormStore,
  getValues,
} from "@modular-forms/solid";
import { confirm } from "@tauri-apps/plugin-dialog";
import Save from "lucide-solid/icons/save";
import Trash2 from "lucide-solid/icons/trash-2";
import WandSparkles from "lucide-solid/icons/wand-sparkles";
import { createEffect } from "solid-js";
import { useView, useWallpapers } from "../../GlobalState";
import {
  addWallpaper,
  applyWallpaper,
  removeWallpaper,
} from "../../lib/binding/emit_wallpaper";
import type {
  Filter,
  Wallpaper,
  WallpaperSource,
} from "../../lib/binding/payload_config";
import { buttonClass } from "../ui";
import ApplicationField from "./ApplicationField";
import FilterFields from "./FilterFields";
import WallpaperNameField from "./NameField";
import OpacityField from "./OpacityField";
import SourceField from "./SourceField";

export type WallpaperForm = {
  name: string;
  applicationPath: string;
  filters: Filter[];
  source: WallpaperSource;
  opacity: number;
};

const DEFAULT_WALLPAPER_VALUE: WallpaperForm = {
  name: "",
  applicationPath: "",
  filters: [
    {
      type: "WindowName",
      name: "",
      strategy: "Contains" as const,
    },
  ],
  source: {
    type: "Picture",
    location: "",
  },
  opacity: 0.2,
};

export default function WallpaperForm(props: {
  id: string;
  wallpaper: Wallpaper | undefined;
  setDirty: (dirty: boolean) => void;
}) {
  const { wallpaper, id, setDirty } = props;
  let isNew = wallpaper === undefined;

  const form = createFormStore<WallpaperForm>({
    initialValues: wallpaper || DEFAULT_WALLPAPER_VALUE,
  });
  const [, setWallpapers] = useWallpapers();
  const [, setView] = useView();

  createEffect(() => {
    setDirty(form.internal.dirty.get());
  });

  const handleSubmit: SubmitHandler<WallpaperForm> = (newWallpaper, event) => {
    if ((event.submitter as HTMLButtonElement).value === "save") {
      setWallpapers((wallpapers) => {
        wallpapers[id] = newWallpaper;
        return wallpapers;
      });

      setView({ type: "home" });
    }

    handleApply(newWallpaper);
  };

  const handleApply = (newWallpaper: WallpaperForm) => {
    if (isNew) {
      addWallpaper({
        id,
        wallpaper: newWallpaper,
      });

      isNew = false;
    } else {
      const changedValues = getValues(form, { shouldDirty: true });

      applyWallpaper({
        ...changedValues,
        filters:
          changedValues.filters !== undefined
            ? newWallpaper.filters
            : undefined,
        source:
          changedValues.source !== undefined ? newWallpaper.source : undefined,
      });
    }
  };

  const deleteWallpaper = async () => {
    if (!(await confirm("本当に壁紙を削除しますか？"))) {
      return;
    }

    setWallpapers((wallpapers) => {
      delete wallpapers[id];
      return wallpapers;
    });
    setView({ type: "home" });

    removeWallpaper(id);
  };

  return (
    <Form of={form} class="space-y-2" onSubmit={handleSubmit}>
      <WallpaperNameField form={form} defaultName={wallpaper?.name} />

      <ApplicationField form={form} />

      <FilterFields form={form} />

      <SourceField form={form} />

      <OpacityField form={form} />

      <div class="my-4 flex gap-2">
        <button
          type="submit"
          value="save"
          class={buttonClass({ withIcon: true })}
        >
          <Save />
          保存する
        </button>

        <button
          type="submit"
          value="try"
          class={buttonClass({ color: "secondary", withIcon: true })}
        >
          <WandSparkles />
          試してみる
        </button>

        <button
          type="button"
          class={buttonClass({ color: "error", withIcon: true })}
          onClick={deleteWallpaper}
        >
          <Trash2 />
          削除する
        </button>
      </div>
    </Form>
  );
}
