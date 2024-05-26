/* @refresh reload */
import { render } from "solid-js/web";

import "./styles.css";
import App from "./App";

document.documentElement.style.colorScheme = "dark";

render(() => <App />, document.getElementById("root") as HTMLElement);
