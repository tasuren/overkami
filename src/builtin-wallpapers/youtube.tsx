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

export default function App(props: { url: string }) {
  const id = extractId(props.url);

  if (id) {
    return (
      <div>
        <iframe
          title="YouTube Video"
          class="border-0 w-screen h-screen object-cover"
          src={makeEmbedUrl(id)}
          allow="accelerometer; autoplay; encrypted-media; gyroscope"
          allowfullscreen
        />
      </div>
    );
  }
  return (
    <div class="w-screen h-screen flex justify-center content-center">
      <div class="text-3xl">YouTubeの動画URLが正しくありません。</div>
    </div>
  );
}
