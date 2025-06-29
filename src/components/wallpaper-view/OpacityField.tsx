import {
  Field,
  type FormStore,
  setValue,
  toCustom,
} from "@modular-forms/solid";
import { fieldClass, inputClass } from "../ui";
import type { WallpaperForm } from "./WallpaperForm";

export default function OpacityField(props: {
  form: FormStore<WallpaperForm>;
}) {
  const { form } = props;

  const { base, error } = fieldClass();

  return (
    <Field
      of={form}
      name="opacity"
      type="number"
      transform={toCustom<number>(
        (value) => {
          if (value) {
            return value / 100;
          }

          return value;
        },
        { on: "input" },
      )}
    >
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
            value={field.value ? field.value * 100 : undefined}
            class={inputClass({ class: "w-24" })}
            onChange={(e) => {
              if (e.target.value === "") {
                e.target.value = "0";
                setValue(form, "opacity", 0);
              }
            }}
          />
          <div class={error()}>{field.error}</div>
        </div>
      )}
    </Field>
  );
}
