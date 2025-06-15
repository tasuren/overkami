import {
  Field,
  type FieldElementProps,
  type FieldStore,
  type FormStore,
  required,
  setValue,
} from "@modular-forms/solid";
import ChevronDown from "lucide-solid/icons/chevron-down";
import RefreshCcw from "lucide-solid/icons/refresh-ccw";
import { For, Show, createSignal, onMount, splitProps } from "solid-js";
import {
  type ApplicationWindow,
  getApplicationWindows,
} from "../../lib/binding";
import {
  fieldClass,
  iconButtonClass,
  iconClass,
  selectClass,
  textMutedClass,
} from "../ui";
import type { WallpaperForm } from "./WallpaperForm";

export default function ApplicationField(props: {
  form: FormStore<WallpaperForm>;
  defaultAppPath?: string;
}) {
  const { form, defaultAppPath } = props;
  const { base, error } = fieldClass();

  return (
    <Field
      of={form}
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
            <Show when={field.value || defaultAppPath}>
              {field.value || defaultAppPath}
            </Show>
          </div>

          <div class={error()}>{field.error}</div>
        </div>
      )}
    </Field>
  );
}

function ApplicationSelect(
  props: FieldElementProps<WallpaperForm, "application.path"> & {
    form: FormStore<WallpaperForm>;
    field: FieldStore<WallpaperForm, "application.path">;
  },
) {
  const [{ field, form }, selectProps] = splitProps(props, ["field", "form"]);
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

  const { base, select, chevron } = selectClass();

  return (
    <div class="flex items-center gap-2 w-96">
      <div class={base()}>
        <select
          {...selectProps}
          class={select({ disabled: options() === undefined })}
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
