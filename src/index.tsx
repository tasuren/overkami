/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";
import PictureApp from "./builtin-wallpapers/picture";
import VideoApp from "./builtin-wallpapers/video";
import YouTubeApp from "./builtin-wallpapers/youtube";

const root = document.getElementById("root") as HTMLElement;

const params = new URLSearchParams(window.location.search);
const wallpaperType = params.get("wallpaper");

if (wallpaperType) {
  const location = params.get("location");

  if (!location) {
    root.innerHTML =
      '<h1 class="text-2xl">パスが指定されていません。このエラーはバグによるものです。</h1>';
  } else {
    switch (wallpaperType) {
      case "picture":
        render(() => <PictureApp path={location} />, root);
        break;
      case "video":
        render(() => <VideoApp path={location} />, root);
        break;
      case "youtube":
        render(() => <YouTubeApp url={location} />, root);
        break;
      default:
        root.innerHTML =
          '<h1 class="text-2xl">不明な壁紙の種類です。これは恐らくバグです。</h1>';
        break;
    }
  }
} else {
  render(() => <App />, root);
}
