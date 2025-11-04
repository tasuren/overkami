import { cl } from "../../lib/utils";

function extractId(urlRaw: string): string | null {
  const url = new URL(urlRaw);

  if (url.host === "youtu.be") {
    return url.pathname;
  }
  if (url.host === "www.youtube.com" || url.host === "youtube.com") {
    return url.searchParams.get("v");
  }

  return null;
}

function makeEmbedUrl(id: string) {
  return `https://www.youtube.com/embed/${id}?mute=1&autoplay=1&loop=1&playlist=${id}&controls=0&disablekb=1`;
}

// 参考: https://stackoverflow.com/a/79395341/14113394
export default function YouTubeEmbed(props: {
  url: string;
  className?: string;
}) {
  const className = props.className ?? "";
  const id = extractId(props.url);

  if (id === null) {
    return (
      <div class="w-full h-full flex justify-center content-center">
        <div class="text-3xl">YouTubeの動画URLが正しくありません。</div>
      </div>
    );
  }

  return (
    <div
      class={cl(
        "w-full h-full",
        "@container flex justify-center items-center",
        "overflow-hidden",
        className,
      )}
    >
      <iframe
        title="YouTube Video"
        class="border-0 pointer-events-none"
        style={{
          "min-width": "max(calc(100cqh * 16 / 9), 100vw)",
          "aspect-ratio": "16 / 9",
        }}
        src={makeEmbedUrl(id)}
        allow="accelerometer; autoplay; encrypted-media; gyroscope"
        allowfullscreen
      />
    </div>
  );
}
