import { emit, listen } from "@/utils.ts";
import { Info } from "@/types.ts";
import { createSignal, For, onMount, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";

export default function PluginSelect() {
  const [services, setServices] = createSignal<Info[]>([]);
  const navigate = useNavigate()

  onMount(() => emit("refresh"))
  listen("plugins", ({ payload: plugins }) => setServices(plugins.sort((p1, p2) => p1.name.localeCompare(p2.name))));

  listen("init_result", success => { if (success.payload) navigate("/folders") })

  return (
    <main class="container">
      <h1>Welcome to Tauri + Solid + Lua</h1>
      <For each={services()}>
        {({ name, description, author, icon_url, filename }) => (
          <div onclick={() => emit("init", filename)}>
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
