import {
  Field,
  type FieldElementProps,
  type FieldStore,
  type FormStore,
  getValue,
  required,
  setValue,
} from "@modular-forms/solid";
import { basename } from "@tauri-apps/api/path";
import { type DialogFilter, open } from "@tauri-apps/plugin-dialog";
import ChevronDown from "lucide-solid/icons/chevron-down";
import { createSignal, onMount } from "solid-js";
import type { WallpaperSource } from "../../lib/binding/payload_config";
import { fieldClass, iconClass, inputClass, selectClass } from "../ui";
import type { WallpaperForm } from "./WallpaperForm";

export default function SourceField(props: { form: FormStore<WallpaperForm> }) {
  const { form } = props;

  const [type, setType] = createSignal<WallpaperSource["type"]>("Picture");

  const selectFile = async () => {
    let filter: DialogFilter | undefined;

    switch (getValue(form, "source.type")) {
      case "Picture":
        filter = {
          name: "画像",
          extensions: ["png", "jpg", "jpeg", "gif", "webp", "avif", "bmp"],
        };
        break;
      case "Video":
        filter = {
          name: "動画",
          extensions: ["mp4", "webm", "avi", "mov", "flv"],
        };
        break;
      case "LocalWebPage":
        filter = {
          name: "HTML",
          extensions: ["html", "htm"],
        };
        break;
    }

    if (filter === undefined) return;

    const path = await open({
      multiple: false,
      filters: [filter],
    });

    if (path !== null) {
      setValue(form, "source.location", path);
    }
  };

  return (
    <div>
      <div class="mb-2">壁紙</div>

      <SourceTypeField
        form={form}
        onChange={(a) => {
          console.log(a);
          setType(a);
        }}
      />

      <Field
        of={form}
        name="source.location"
        validate={[required("壁紙に使うファイルを指定してください。")]}
      >
        {(field, props) => {
          const { base, error } = fieldClass();
          let fileName = field.value;

          let buttonElement!: HTMLButtonElement;
          onMount(async () => {
            if (fileName) {
              fileName = await basename(fileName);
            } else {
              fileName = "クリックでファイルを選択";
            }

            buttonElement.innerText = fileName;
          });

          let title: string | undefined;
          switch (type()) {
            case "Picture":
              title = "壁紙に使う画像ファイル";
              break;
            case "Video":
              title = "壁紙に使う動画ファイル";
              break;
            case "YouTube":
              title = "壁紙に使うYouTubeの動画URL";
              break;
          }

          if (type() === "YouTube") {
            return (
              <div class={base()}>
                <label for={props.name} class="text-sm">
                  {title}
                </label>

                <input
                  {...props}
                  id={props.name}
                  class={inputClass({
                    class: "text-left font-mono",
                  })}
                  type="url"
                  value={field.value}
                />

                <div class={error()}>{field.error}</div>
              </div>
            );
          }
          return (
            <div class={base()}>
              <label for={props.name} class="text-sm">
                {title}
              </label>
              <input
                {...props}
                id={props.name}
                type="text"
                value={field.value}
                hidden
              />

              <button
                type="button"
                class={inputClass({
                  file: true,
                  class: "text-left font-mono overflow-hidden",
                })}
                onClick={selectFile}
                ref={buttonElement}
              />

              <div class={error()}>{field.error}</div>
            </div>
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
  const { base, select, chevron, optionClass } = selectClass();

  return (
    <div class={base({ size: "sm" })}>
      <select
        {...fieldProps}
        class={select()}
        id={id}
        onChange={() => onChange(field.value || "Picture")}
      >
        <option class={optionClass()} value="Picture" selected>
          画像
        </option>
        <option class={optionClass()} value="Video">動画</option>
        <option class={optionClass()} value="YouTube">YouTube</option>
      </select>

      <span class={chevron()}>
        <ChevronDown class={iconClass()} />
      </span>
    </div>
  );
}
