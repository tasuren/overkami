import YouTubeEmbed from "../components/wallpaper/YouTubeEmbed";

export default function App(props: { url: string }) {
  return <YouTubeEmbed url={props.url} />;
}
