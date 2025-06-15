/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";

const params = new URLSearchParams(window.location.search);

render(() => <App />, document.getElementById("root") as HTMLElement);
