import { invoke } from "./utils.ts";
import "./App.css";
import { Info } from "./types.ts";
import { createSignal, For } from "solid-js";

function App() {
  const [services, setServices] = createSignal<Info[]>([]);

  invoke("get_plugins").then(setServices);

  return (
    <main class="container">
      <h1>Welcome to Tauri + Solid + Lua</h1>
      <For each={services()}>
        {({ name, description, author }) => (
          <div>
            <h2>{name}</h2>
            <p>Description: {description}</p>
            <span>Written by: {author}</span>
          </div>
        )}
      </For>
    </main>
  );
}

export default App;
