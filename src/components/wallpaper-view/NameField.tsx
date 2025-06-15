import {
  Field,
  type FormStore,
  maxLength,
  minLength,
  required,
} from "@modular-forms/solid";
import { fieldClass, textInputClass } from "../ui";
import type { WallpaperForm } from "./WallpaperForm";

export default function WallpaperNameField(props: {
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
