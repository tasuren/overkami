import type { FieldElementProps } from "@modular-forms/solid";
import {
  Field,
  FieldArray,
  type FieldStore,
  type FormStore,
  insert,
  maxLength,
  minLength,
  required,
  setValue,
} from "@modular-forms/solid";
import ChevronDown from "lucide-solid/icons/chevron-down";
import RefreshCcw from "lucide-solid/icons/refresh-ccw";
import { For, Show, createSignal, onMount, splitProps } from "solid-js";
import {
  type ApplicationWindow,
  STRING_FILTER_STRATEGIES,
  type StringFilterStrategy,
  getApplicationWindows,
} from "../../lib/binding";
import {
  fieldClass,
  iconButtonClass,
  iconClass,
  selectClass,
  textInputClass,
  textMutedClass,
} from "../ui";
import type { WallpaperForm } from "./WallpaperForm";

export function WallpaperNameField(props: {
  defaultName?: string;
  form: FormStore<WallpaperForm>;
}) {
  const { defaultName, form } = props;
  const { base, error } = fieldClass();

  return (
    <Field
      of={form}
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
            value={defaultName}
            placeholder="壁紙の設定名を入力"
            class={textInputClass({ class: "w-96" })}
          />
          <div class={error()}>{field.error}</div>
        </div>
      )}
    </Field>
  );
}

export function ApplicationField(props: {
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
      <div class={base({ class: "w-5/6" })}>
        <select
          {...selectProps}
          class={select({ class: "w-full", disabled: options() === undefined })}
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

export function FilterFields(props: {
  form: FormStore<WallpaperForm>;
}) {
  const { form } = props;
  const { base, error } = fieldClass();

  onMount(() => {
    insert(form, "filters", {
      value: {
        type: "WindowName",
        name: "",
        strategy: "Contains",
      },
    });
  });

  return (
    <div>
      <div class="mb-2">ウィンドウの絞り込み</div>

      <FieldArray of={form} name="filters">
        {(fieldArray) => (
          <For each={fieldArray.items || [0]}>
            {(_, index) => (
              <div>
                <Field of={form} name={`${fieldArray.name}.${index()}.name`}>
                  {(field, props) => (
                    <div class={base()}>
                      <label for={field.name} class={textMutedClass()}>
                        壁紙をつけるウィンドウの名前
                      </label>
                      <input
                        {...props}
                        type="text"
                        class={textInputClass({ class: "w-96" })}
                      />
                      <div class={error()}>{field.error}</div>
                    </div>
                  )}
                </Field>

                <Field
                  of={form}
                  name={`${fieldArray.name}.${index()}.strategy`}
                  validate={[
                    required("ウィンドウの絞り込み方法を選択してください。"),
                  ]}
                >
                  {(field, props) => (
                    <div class={base()}>
                      <label for={field.name} class={textMutedClass()}>
                        ウィンドウ名の絞り込み方法
                      </label>
                      <WindowNameFilterSelect
                        {...props}
                        name={field.name}
                        value={field.value || "Contains"}
                      />
                      <div class={error()}>{field.error}</div>
                    </div>
                  )}
                </Field>
              </div>
            )}
          </For>
        )}
      </FieldArray>
    </div>
  );
}

export default function WindowNameFilterSelect(
  props: FieldElementProps<WallpaperForm, `filters.${number}.strategy`> & {
    name: string;
    value: StringFilterStrategy;
  },
) {
  const { base, select, chevron } = selectClass();

  return (
    <div class={base({ class: "w-96" })}>
      <select class={select({ class: "w-full" })} {...props}>
        <For each={Object.entries(STRING_FILTER_STRATEGIES)}>
          {([value, label]) => (
            <option value={value} selected={props.value === value}>
              {label}
            </option>
          )}
        </For>
      </select>

      <span class={chevron()}>
        <ChevronDown class={iconClass()} />
      </span>
    </div>
  );
}
