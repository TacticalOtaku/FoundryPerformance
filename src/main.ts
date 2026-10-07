// только латиница и кириллица: лаунчер работает без сети, лишние наборы не нужны
import "./styles/fonts.css";
import "./styles/tactile/tokens.css";
import "./styles/base.css";
import { mount } from "svelte";
import App from "./App.svelte";

const app = mount(App, { target: document.getElementById("app")! });

export default app;
