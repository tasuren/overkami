import {
  Form,
  type SubmitHandler,
  createFormStore,
} from "@modular-forms/solid";
import { useView, useWallpapers } from "../../GlobalState";
import type { Filter, Wallpaper, WallpaperSource } from "../../lib/binding";
import { buttonClass } from "../ui";
import { ApplicationField, FilterFields, WallpaperNameField } from "./Field";

export type WallpaperForm = {
  name: string;
  application: {
    name: string | undefined;
    path: string;
  };
  filters: Filter[];
  source: WallpaperSource;
};

export function useWallpaperForm() {
  return createFormStore<WallpaperForm>();
}

export default function WallpaperForm(props: {
  wallpaper: Wallpaper | undefined;
}) {
  const { wallpaper } = props;

  const form = useWallpaperForm();
  const [, setWallpapers] = useWallpapers();
  const [, setView] = useView();

  const handleSubmit: SubmitHandler<WallpaperForm> = (values) => {
    setWallpapers((prev) => [...prev, values]);
    setView({ type: "home" });
  };

  return (
    <Form of={form} class="p-2 space-y-2" onSubmit={handleSubmit}>
      <WallpaperNameField form={form} defaultName={wallpaper?.name} />

      <ApplicationField
        form={form}
        defaultAppPath={wallpaper?.application?.path}
      />

      <FilterFields form={form} />

      <button type="submit" class={buttonClass({ class: "ml-auto" })}>
        保存
      </button>
    </Form>
  );
}
