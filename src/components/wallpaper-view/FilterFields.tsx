import type { FieldElementProps } from "@modular-forms/solid";
import {
  Field,
  FieldArray,
  type FormStore,
  insert,
  required,
} from "@modular-forms/solid";
import ChevronDown from "lucide-solid/icons/chevron-down";
import { For, onMount } from "solid-js";
import {
  STRING_FILTER_STRATEGIES,
  type StringFilterStrategy,
} from "../../lib/binding";
import {
  fieldClass,
  iconClass,
  selectClass,
  textInputClass,
  textMutedClass,
} from "../ui";
import type { WallpaperForm } from "./WallpaperForm";

export default function FilterFields(props: {
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
                <Field of={form} name={`${fieldArray.name}.${index()}.type`}>
                  {(_, props) => (
                    <input {...props} type="text" value="WindowName" hidden />
                  )}
                </Field>

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

function WindowNameFilterSelect(
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
