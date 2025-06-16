/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";
import PictureApp from "./builtin-wallpapers/picture";

const root = document.getElementById("root") as HTMLElement;

const params = new URLSearchParams(window.location.search);
const wallpaperType = params.get("wallpaper");

if (wallpaperType) {
  const path = params.get("path");

  if (!path) {
    root.innerHTML =
      '<h1 class="text-2xl">パスが指定されていません。このエラーはバグによるものです。</h1>';
  } else {
    switch (wallpaperType) {
      case "picture":
        render(() => <PictureApp path={path} />, root);
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
