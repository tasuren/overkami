import "./styles.css";
import App from "./App.svelte";

import { allComponents, provideFluentDesignSystem, baseLayerLuminance, StandardLuminance } from "@fluentui/web-components";


provideFluentDesignSystem()
  .register(allComponents);
baseLayerLuminance.setValueFor(document.documentElement, StandardLuminance.DarkMode);


const app = new App({
  target: document.getElementById("app") as HTMLElement,
});

export default app;
