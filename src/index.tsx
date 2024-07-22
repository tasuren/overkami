/* @refresh reload */
import { render } from "solid-js/web";

import "./styles.css";

const app = location.pathname.endsWith("wallpaper")
    ? (await import("./wallpaper/App")).default
    : (await import("./App")).default;
render(app, document.getElementById("root") as HTMLElement);

