import { Field, type FormStore } from "@modular-forms/solid";
import { fieldClass, inputClass } from "../ui";
import type { WallpaperForm } from "./WallpaperForm";

export default function OpacityField(props: {
  form: FormStore<WallpaperForm>;
  defaultOpacity: number;
}) {
  const { form, defaultOpacity } = props;

  const { base, error } = fieldClass();

  return (
    <Field of={form} name="opacity">
      {(field, props) => (
        <div class={base()}>
          <label for={props.name}>壁紙の不透明度</label>
          <input
            {...props}
            id={props.name}
            type="number"
            min={0}
            max={100}
            step={1}
            value={defaultOpacity}
            class={inputClass({ class: "w-24" })}
          />
          <div class={error()}>{field.error}</div>
        </div>
      )}
    </Field>
  );
}
