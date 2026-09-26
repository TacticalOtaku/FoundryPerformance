import "@fontsource/geologica/400.css";
import "@fontsource/geologica/500.css";
import "@fontsource/geologica/800.css";
import "@fontsource/martian-mono/400.css";
import "@fontsource/martian-mono/500.css";
import "./styles/tokens.css";
import "./styles/base.css";
import { mount } from "svelte";
import App from "./App.svelte";

const app = mount(App, { target: document.getElementById("app")! });

export default app;
