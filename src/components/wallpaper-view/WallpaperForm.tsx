import {
  createFormStore,
  Form,
  getValues,
  type SubmitHandler,
} from "@modular-forms/solid";
import { confirm } from "@tauri-apps/plugin-dialog";
import Save from "lucide-solid/icons/save";
import Trash2 from "lucide-solid/icons/trash-2";
import WandSparkles from "lucide-solid/icons/wand-sparkles";
import { createEffect, onCleanup } from "solid-js";
import { useConfig, useView, useWallpapers } from "../../GlobalState";
import { saveConfig } from "../../lib/binding/command_config";
import {
  addWallpaper,
  applyWallpaper,
  removeWallpaper,
} from "../../lib/binding/command_wallpaper";
import type {
  Filter,
  Wallpaper,
  WallpaperSource,
} from "../../lib/binding/payload_config";
import type { ApplyWallpaper } from "../../lib/binding/payload_wallpaper";
import { buttonClass } from "../ui";
import ApplicationField from "./ApplicationField";
import FilterFields from "./FilterFields";
import WallpaperNameField from "./NameField";
import OpacityField from "./OpacityField";
import SourceField from "./SourceField";

export type WallpaperForm = {
  name: string;
  applicationName: string;
  filters: Filter[];
  source: WallpaperSource;
  opacity: number;
};

const DEFAULT_WALLPAPER_VALUE: WallpaperForm = {
  name: "",
  applicationName: "",
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

function filterObject<K extends string, V>(
  obj: Partial<Record<K, V>>,
  filter: (key: K, value: V) => boolean,
): Partial<Record<K, V>> {
  return Object.fromEntries(
    Object.entries(obj).filter(([key, value]) => filter(key as K, value as V)),
  ) as Partial<Record<K, V>>;
}

function mapObj(
  from: ApplyWallpaper,
  to: ApplyWallpaper,
  filter: (
    key: keyof ApplyWallpaper,
    value: ApplyWallpaper[keyof ApplyWallpaper],
  ) => boolean,
): ApplyWallpaper {
  return Object.fromEntries(
    Object.entries(from)
      .filter(([key, value]) => filter(key as keyof ApplyWallpaper, value))
      .map(([key, _]) => [key, to[key as keyof ApplyWallpaper]]),
  );
}

function mapObjUndefined(
  from: ApplyWallpaper,
  filter: (
    key: keyof ApplyWallpaper,
    value: ApplyWallpaper[keyof ApplyWallpaper],
  ) => boolean,
): ApplyWallpaper {
  return Object.fromEntries(
    Object.entries(from)
      .filter(([key, value]) => filter(key as keyof ApplyWallpaper, value))
      .map(([key, _]) => {
        return [key, undefined];
      }),
  );
}

export default function WallpaperForm(props: {
  id: string;
  wallpaper: Wallpaper | undefined;
  setDirty: (dirty: boolean) => void;
}) {
  let { wallpaper, id, setDirty } = props;
  let isNew = wallpaper === undefined;
  const initialValues = wallpaper || DEFAULT_WALLPAPER_VALUE;

  const form = createFormStore<WallpaperForm>({
    initialValues,
  });

  const [, setWallpapers] = useWallpapers();
  const [, setView] = useView();
  const [config] = useConfig();

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
      saveConfig(config());
    }

    handleApply(newWallpaper);
  };

  let undo: ApplyWallpaper = {};

  const handleApply = (newWallpaper: WallpaperForm) => {
    if (isNew) {
      addWallpaper(id, newWallpaper);

      isNew = false;
      wallpaper = newWallpaper;
    } else {
      const changedValues = getValues(form, { shouldDirty: true });
      let payload: ApplyWallpaper = {
        ...changedValues,
        filters:
          changedValues.filters !== undefined
            ? newWallpaper.filters
            : undefined,
        source:
          changedValues.source !== undefined ? newWallpaper.source : undefined,
      };

      undo = Object.assign(
        undo,
        mapObj(payload, initialValues, (_, value) => value !== undefined),
      );

      // Include values that have been changed twice and then reverted back
      // to their original state.
      // Simply obtaining the fields that have been changed from their initial state
      // using `shouldDirty: true` does not include fieldsthat were changed once,
      // the Try button was pressed, but were reverted back to their original state.
      // Therefore, include fields that have been changed twice.
      const reverted = filterObject(
        undo,
        (key, _) => !(key in payload) || payload[key] === undefined,
      ) as ApplyWallpaper;
      payload = Object.assign(payload, reverted);

      applyWallpaper(id, payload);

      // `reverted` fields are already used and reverted fields on `undo`
      // are not needed anymore. So we can remove them.
      undo = Object.assign(
        undo,
        mapObjUndefined(reverted, (_, value) => value !== undefined),
      );
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
    saveConfig(config());
  };

  onCleanup(() => {
    if (form.dirty) {
      // Reset wallpaper state.
      // Wallpaper state may be dirty if the user use "Try" button.
      applyWallpaper(id, undo);
    }
  });

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
