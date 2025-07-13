import {
  Field,
  type FieldElementProps,
  type FieldStore,
  type FormStore,
  required,
} from "@modular-forms/solid";
import { basename } from "@tauri-apps/api/path";
import ChevronDown from "lucide-solid/icons/chevron-down";
import RefreshCcw from "lucide-solid/icons/refresh-ccw";
import { createResource, For, Show, splitProps } from "solid-js";
import { getApplicationWindows } from "../../lib/binding/command_os";
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
}) {
  const { form } = props;
  const { base, error } = fieldClass();

  return (
    <Field
      of={form}
      name="applicationName"
      validate={[required("壁紙を適用するアプリを選択してください。")]}
    >
      {(field, props) => (
        <div class={base()}>
          <label for={props.name}>壁紙を適用するアプリ</label>

          <ApplicationSelect {...props} field={field} />

          <div
            class={textMutedClass({
              class: [
                "overflow-hidden",
                "font-mono text-sm cursor-text select-all",
              ],
            })}
          >
            <Show when={field.value}>{field.value}</Show>
          </div>

          <div class={error()}>{field.error}</div>
        </div>
      )}
    </Field>
  );
}

function ApplicationSelect(
  props: FieldElementProps<WallpaperForm, "applicationName"> & {
    field: FieldStore<WallpaperForm, "applicationName">;
  },
) {
  const [{ field }, selectProps] = splitProps(props, ["field"]);

  // Retrieve the list of application windows for the select
  const [applicationWindows, { mutate, refetch }] = createResource(async () => {
    return await getApplicationWindows();
  });

  const reloadApplicationWindows = async () => {
    mutate(undefined);
    refetch();
  };

  const [currentFileName] = createResource(async () =>
    field.value ? await basename(field.value) : undefined,
  );

  const { base, select, chevron } = selectClass();

  return (
    <div class="flex items-center gap-2 w-96">
      <div class={base()}>
        <select
          {...selectProps}
          class={select({ disabled: applicationWindows() === undefined })}
          id={selectProps.name}
          disabled={applicationWindows() === undefined}
          value={field.value}
        >
          <option value="" disabled>
            <Show
              when={applicationWindows() === undefined}
              fallback="アプリを選択"
            >
              読み込み中
            </Show>
          </option>

          <For each={applicationWindows()}>
            {(option) => (
              <option
                value={option.name}
                selected={field.value === option.name}
              >
                {option.windowTitle || option.name}
              </option>
            )}
          </For>

          <Show
            when={
              currentFileName() &&
              applicationWindows()?.find((app) => app.name === field.value) ===
                undefined
            }
          >
            <option value={field.value} selected>
              {currentFileName()}
            </option>
          </Show>
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
