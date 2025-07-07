import { convertFileSrc } from "@tauri-apps/api/core";

export default function App(props: { path: string }) {
  const url = convertFileSrc(props.path);

  return (
    <div class="w-screen h-screen">
      <img
        class="w-full h-full object-cover"
        src={url}
        alt="画像のロードに失敗しました。"
      />
    </div>
  );
}
