import { emit, listen, invoke, unlisten } from "@/logic/backend";
import { Info } from "@/types/data";
import { createSignal, Index, onMount, Show } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { Portal } from "solid-js/web";
import { createStore, reconcile } from "solid-js/store";

export default function PluginSelect() {
  const navigate = useNavigate()

  const [services, setServices] = createStore<Info[]>([]);
  const [loading, setLoading] = createSignal<AbortInfo | undefined>()

  function init(pair: AbortInfo) {
    setLoading(pair)
    emit("init", pair.filename)
  }

  unlisten([
    listen("init_result", () => navigate("/tags"))
  ])()

  // run on app boot
  emit("saved_plugin")

  const refresh = () => invoke("get_plugins").then(plugins => setServices(reconcile(plugins.sort((p1, p2) => p1.name.localeCompare(p2.name)))));
  onMount(() => { refresh() })

  return <>
    <main class="container items-center">
      <Portal>
        <div class="fixed right-0 bottom-0 m-4">
          <button onclick={refresh}>Refresh</button>
        </div>
      </Portal>
      <Show when={!loading()} fallback={<>
        <h1>Now loading: {loading()!.name}</h1>
        <button onClick={() => { emit("abort", loading()!.filename); setLoading() }}>Cancel loading</button>
      </>}>
        <h1>Welcome to Tauri + Solid + Lua</h1>
        <div class="space-y-5">
          <Index each={services}>
            {elem =>
              // Do not destructure elem to retain reactivity
              <div onclick={[init, { name: elem().name, filename: elem().filename }]} class="flex border justify-end p-1.5 cursor-pointer rounded-lg">
                <Show when={elem().icon_url}>
                  <img src={elem().icon_url} class="w-10 mr-4" />
                </Show>
                <div>
                  <h2>{elem().name}</h2>
                  <p>Description: {elem().description}</p>
                  <span>Written by: {elem().author}</span>
                </div>
              </div>
            }
          </Index>
        </div>
      </Show>
    </main >
  </>
}

type AbortInfo = Pick<Info, "name" | "filename">
