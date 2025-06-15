import {
  Field,
  type FieldElementProps,
  type FieldStore,
  type FormStore,
  required,
} from "@modular-forms/solid";
import ChevronDown from "lucide-solid/icons/chevron-down";
import { Show, createSignal } from "solid-js";
import type { WallpaperSource } from "../../lib/binding";
import { fieldClass, iconClass, inputClass, selectClass } from "../ui";
import type { WallpaperForm } from "./WallpaperForm";

export default function SourceField(props: { form: FormStore<WallpaperForm> }) {
  const { form } = props;

  const [type, setType] = createSignal<WallpaperSource["type"]>("Picture");

  return (
    <div>
      <div class="mb-2">壁紙</div>

      <SourceTypeField form={form} onChange={setType} />

      <Field
        of={form}
        name="source.path"
        validate={[required("壁紙に使うHTMLファイルを指定してください。")]}
      >
        {(field, props) => {
          const { base, error } = fieldClass();

          return (
            <Show when={type() === "Picture"}>
              <div class={base()}>
                <label for={props.name} class="text-sm">
                  壁紙のパス
                </label>
                <input
                  {...props}
                  id={props.name}
                  type="file"
                  class={inputClass({ file: true })}
                  placeholder="壁紙のパスを入力してください"
                />
                <div class={error()}>{field.error}</div>
              </div>
            </Show>
          );
        }}
      </Field>
    </div>
  );
}

export function SourceTypeField(props: {
  form: FormStore<WallpaperForm>;
  onChange: (type: WallpaperSource["type"]) => void;
}) {
  const { form, onChange } = props;
  const { base, error } = fieldClass();

  return (
    <Field of={form} name="source.type">
      {(field, props) => (
        <div class={base()}>
          <label for={props.name} class="text-sm">
            壁紙の種類
          </label>
          <SourceTypeSelect
            field={field}
            fieldProps={props}
            id={props.name}
            onChange={onChange}
          />
          <div class={error()}>{field.error}</div>
        </div>
      )}
    </Field>
  );
}

function SourceTypeSelect(props: {
  field: FieldStore<WallpaperForm, "source.type">;
  fieldProps: FieldElementProps<WallpaperForm, "source.type">;
  onChange: (type: WallpaperSource["type"]) => void;
  id: string;
}) {
  const { field, fieldProps, id, onChange } = props;
  const { base, select, chevron } = selectClass();

  return (
    <div class={base({ size: "sm" })}>
      <select
        {...fieldProps}
        class={select()}
        id={id}
        onSelect={() => onChange(field.value || "Picture")}
      >
        <option value="Picture" selected>
          画像
        </option>
      </select>

      <span class={chevron()}>
        <ChevronDown class={iconClass()} />
      </span>
    </div>
  );
}
