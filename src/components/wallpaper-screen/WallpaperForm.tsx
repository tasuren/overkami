import {
  type FieldElementProps,
  type FieldStore,
  createForm,
  maxLength,
  minLength,
  required,
} from "@modular-forms/solid";
import ChevronDown from "lucide-solid/icons/chevron-down";
import RefreshCcw from "lucide-solid/icons/refresh-ccw";
import { For, Show, createSignal } from "solid-js";
import type { EditingWallpaper } from "../../GlobalState";
import {
  type ApplicationWindow,
  getApplicationWindows,
} from "../../lib/binding";
import {
  buttonClass,
  fieldClass,
  iconButtonClass,
  iconClass,
  selectClass,
  textInputClass,
} from "../ui";

type WallpaperForm = {
  name: string;
  targetPath: string;
};

export default function WallpaperForm(props: {
  wallpaper?: EditingWallpaper;
  setName: (name: string) => void;
}) {
  const { wallpaper } = props;
  const [form, { Form, Field }] = createForm<WallpaperForm>();

  return (
    <Form class="space-y-4">
      <Field
        name="name"
        validate={[
          required("壁紙の名前を入力してください。"),
          minLength(2, "壁紙の名前は2文字以上である必要があります。"),
          maxLength(100, "壁紙の名前は100文字以下である必要があります。"),
        ]}
      >
        {(field, props) => (
          <div class={fieldClass()}>
            <label for={props.name}>壁紙の設定名</label>
            <input
              {...props}
              id={props.name}
              type="text"
              value={wallpaper?.name}
              placeholder="壁紙の設定名を入力"
              class={textInputClass({ class: "w-96" })}
            />
          </div>
        )}
      </Field>

      <Field
        name="targetPath"
        validate={[required("壁紙を適用するアプリを選択してください。")]}
      >
        {(field, props) => (
          <ApplicationSelect {...props} wallpaper={wallpaper} field={field} />
        )}
      </Field>

      <button type="submit" class={buttonClass({ class: "ml-auto" })}>
        保存
      </button>
    </Form>
  );
}

function ApplicationSelect(
  props: FieldElementProps<WallpaperForm, "targetPath"> & {
    field: FieldStore<WallpaperForm, "targetPath">;
    wallpaper?: EditingWallpaper;
  },
) {
  const { wallpaper, field } = props;
  const [options, setOptions] = createSignal<ApplicationWindow[]>();

  const loadApplicationWindows = async () => {
    const windows = await getApplicationWindows();
    setOptions(windows);
  };
  loadApplicationWindows();

  const onClick = async () => {
    setOptions(undefined);
    loadApplicationWindows();
  };

  const onSelect = (event: Event) => {
    console.log((event.currentTarget as HTMLSelectElement).value);
  };

  const { base, chevron } = selectClass();

  return (
    <div class={fieldClass()}>
      <label for={props.name}>壁紙を適用するアプリ</label>

      <div class="flex items-center gap-2 w-96">
        <div class="relative w-5/6">
          <select
            {...props}
            class={base({ class: "w-full", disabled: options() === undefined })}
            name={field.name}
            id={field.name}
            disabled={options() === undefined}
            onSelect={onSelect}
          >
            <option value="" disabled selected>
              <Show when={options() === undefined} fallback="アプリを選択">
                読み込み中
              </Show>
            </option>

            <For each={options()}>
              {(option) => (
                <option value={option.path}>
                  {option.windowTitle || option.name || "不明なアプリ"}
                </option>
              )}
            </For>
          </select>

          <span class={chevron()}>
            <ChevronDown class={iconClass()} />
          </span>
        </div>

        <div class="w-1/6">
          <button
            type="button"
            class={iconButtonClass({ class: "block mx-auto" })}
            onClick={onClick}
          >
            <RefreshCcw />
          </button>
        </div>
      </div>

      <div
        class={textInputClass({
          class: "font-mono w-96",
          disabled: true,
        })}
      >
        <Show
          when={field.value || wallpaper?.targetPath}
          fallback="アプリを選択"
        >
          {field.value || wallpaper?.targetPath}
        </Show>
      </div>
    </div>
  );
}
