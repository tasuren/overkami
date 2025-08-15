import { convertFileSrc } from "@tauri-apps/api/core";

export default function App(props: { path: string }) {
  const url = convertFileSrc(props.path);

  return (
    <div class="w-screen h-screen">
      <video
        autoplay
        loop
        muted
        preload="auto"
        src={url}
        class="w-full h-full object-cover"
      />
    </div>
  );
}
