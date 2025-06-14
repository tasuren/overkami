import {
  type FieldElementProps,
  type FieldStore,
  type FormStore,
  type SubmitHandler,
  createForm,
  maxLength,
  minLength,
  required,
  setValue,
} from "@modular-forms/solid";
import ChevronDown from "lucide-solid/icons/chevron-down";
import RefreshCcw from "lucide-solid/icons/refresh-ccw";
import { For, Show, createSignal, onMount, splitProps } from "solid-js";
import { useView, useWallpapers } from "../../GlobalState";
import {
  type ApplicationWindow,
  type Wallpaper,
  getApplicationWindows,
} from "../../lib/binding";
import {
  buttonClass,
  fieldClass,
  iconButtonClass,
  iconClass,
  selectClass,
  textInputClass,
  textMutedClass,
} from "../ui";

type WallpaperForm = {
  name: string;
  application: {
    name: string | undefined;
    path: string;
  };
};

export default function WallpaperForm(props: {
  wallpaper: Wallpaper | undefined;
}) {
  const { wallpaper } = props;
  const [form, { Form, Field }] = createForm<WallpaperForm>();
  const [wallpapers, setWallpapers] = useWallpapers();
  const [, setView] = useView();

  const { base, error } = fieldClass();

  const handleSubmit: SubmitHandler<WallpaperForm> = (values) => {
    const items = wallpapers();
    items.push(values);
    setWallpapers(items);
    setView({ type: "wallpaper", wallpaper: values });
  };

  return (
    <Form class="space-y-4" onSubmit={handleSubmit}>
      <Field
        name="name"
        validate={[
          required("壁紙の名前を入力してください。"),
          minLength(2, "壁紙の名前は2文字以上である必要があります。"),
          maxLength(100, "壁紙の名前は100文字以下である必要があります。"),
        ]}
      >
        {(field, props) => (
          <div class={base()}>
            <label for={props.name}>壁紙の設定名</label>
            <input
              {...props}
              id={props.name}
              type="text"
              value={wallpaper?.name}
              placeholder="壁紙の設定名を入力"
              class={textInputClass({ class: "w-96" })}
            />
            <div class={error()}>{field.error}</div>
          </div>
        )}
      </Field>

      <Field
        name="application.path"
        validate={[required("壁紙を適用するアプリを選択してください。")]}
      >
        {(field, props) => (
          <div class={base()}>
            <label for={props.name}>壁紙を適用するアプリ</label>

            <ApplicationSelect {...props} form={form} field={field} />

            <div
              class={textMutedClass({
                class: [
                  "overflow-hidden",
                  "font-mono text-sm cursor-text select-all",
                ],
              })}
            >
              <Show
                when={field.value || wallpaper?.application?.name}
                fallback=""
              >
                {field.value || wallpaper?.application?.path}
              </Show>
            </div>

            <div class={error()}>{field.error}</div>
          </div>
        )}
      </Field>

      <button type="submit" class={buttonClass({ class: "ml-auto" })}>
        保存
      </button>
    </Form>
  );
}

function ApplicationSelect(
  raw: FieldElementProps<WallpaperForm, "application.path"> & {
    form: FormStore<WallpaperForm>;
    field: FieldStore<WallpaperForm, "application.path">;
  },
) {
  const [{ field, form }, props] = splitProps(raw, ["field", "form"]);
  const [options, setOptions] = createSignal<ApplicationWindow[]>();

  const onSelect = () => {
    for (const option of options() || []) {
      if (option.path !== field.value) continue;

      setValue(form, "application.name", option.name);
      break;
    }
  };

  // Retrieve the list of application windows for the select
  const loadApplicationWindows = async () => {
    const windows = await getApplicationWindows();
    setOptions(windows);
  };

  onMount(() => {
    loadApplicationWindows();
  });

  const reloadApplicationWindows = async () => {
    setOptions(undefined);
    loadApplicationWindows();
  };

  const { base, chevron } = selectClass();

  return (
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
          onClick={reloadApplicationWindows}
        >
          <RefreshCcw />
        </button>
      </div>
    </div>
  );
}
