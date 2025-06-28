import {
  Form,
  type SubmitHandler,
  createFormStore,
} from "@modular-forms/solid";
import Save from "lucide-solid/icons/save";
import Trash2 from "lucide-solid/icons/trash-2";
import { useView, useWallpapers } from "../../GlobalState";
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
  application: {
    name: string | null;
    path: string;
  };
  filters: Filter[];
  source: WallpaperSource;
  opacity: string;
};

export function useWallpaperForm() {
  return createFormStore<WallpaperForm>();
}

export default function WallpaperForm(props: {
  id: string;
  wallpaper: Wallpaper | undefined;
}) {
  const { wallpaper, id } = props;

  const form = useWallpaperForm();
  const [, setWallpapers] = useWallpapers();
  const [, setView] = useView();

  const handleSubmit: SubmitHandler<WallpaperForm> = (values) => {
    const newWallpaper: Wallpaper = {
      ...values,
      opacity: Number.parseFloat(values.opacity) / 100,
    };

    // If no wallpaper is provided, we are creating a new one.
    setWallpapers((wallpapers) => {
      wallpapers[id] = newWallpaper;
      return wallpapers;
    });

    setView({ type: "home" });
  };

  const deleteWallpaper = () => {
    setWallpapers((wallpapers) => {
      delete wallpapers[id];
      return wallpapers;
    });
    setView({ type: "home" });
  };

  return (
    <Form of={form} class="space-y-2" onSubmit={handleSubmit}>
      <WallpaperNameField form={form} defaultName={wallpaper?.name} />

      <ApplicationField
        form={form}
        defaultAppPath={wallpaper?.application?.path}
      />

      <FilterFields form={form} defaultFilters={wallpaper?.filters} />

      <SourceField
        form={form}
        defaultSourcePath={wallpaper?.source?.location}
      />

      <OpacityField
        form={form}
        defaultOpacity={(wallpaper?.opacity || 0.3) * 100}
      />

      <div class="my-4 flex gap-2">
        <button type="submit" class={buttonClass({ withIcon: true })}>
          <Save />
          保存
        </button>

        <button
          type="button"
          class={buttonClass({ color: "error", withIcon: true })}
          onClick={deleteWallpaper}
          hidden={wallpaper === undefined}
        >
          <Trash2 />
          削除
        </button>
      </div>
    </Form>
  );
}
