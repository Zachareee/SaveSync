import { emit, listen } from "@/utils.ts";
import { Info } from "@/types.ts";
import { createSignal, For, Match, onMount, Show, Switch } from "solid-js";
import { useNavigate } from "@solidjs/router";

export default function PluginSelect() {
  const [services, setServices] = createSignal<Info[]>([]);
  const [loading, setLoading] = createSignal("")
  const navigate = useNavigate()

  onMount(() => emit("refresh"))
  listen("plugins", ({ payload: plugins }) => setServices(plugins.sort((p1, p2) => p1.name.localeCompare(p2.name))));

  function init({ name, filename }: Info) {
    setLoading(name)
    emit("init", filename)
  }

  listen("init_result", ({ payload }) => {
    if (loading() && payload) navigate("/folders")
    else setLoading("")
  })

  return (
    <main class="container">
      <Switch>
        <Match when={!loading()}>
          <h1>Welcome to Tauri + Solid + Lua</h1>
          <For each={services()}>
            {({ name, description, author, icon_url, filename }) => (
              <div onclick={[init, { name, filename }]}>
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
        </Match>
        <Match when={loading()}>
          <h1>Now loading: {loading()}</h1>
          <button onClick={() => setLoading("")}>Cancel loading</button>
        </Match>
      </Switch>
    </main>
  );
}
