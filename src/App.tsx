import { invoke } from "./utils.ts";
import "./App.css";
import { Info } from "./types.ts";
import { createSignal, For, Show } from "solid-js";

function App() {
  const [services, setServices] = createSignal<Info[]>([]);

  invoke("get_plugins").then(setServices);

  return (
    <main class="container">
      <h1>Welcome to Tauri + Solid + Lua</h1>
      <For each={services()}>
        {({ name, description, author, icon_url }) => (
          <div>
            <h2>{name}</h2>
            <p>Description: {description}</p>
            <p>Written by: {author}</p>
            <p>
              <Show when={icon_url} fallback={"No icon"}>
                Icon url: {icon_url!}
              </Show>
            </p>
          </div>
        )}
      </For>
    </main>
  );
}

export default App;
