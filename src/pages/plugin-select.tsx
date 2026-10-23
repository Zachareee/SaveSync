import { map, mapValues, pick, sortBy } from "lodash"
import { createSignal, Index, onMount, Show } from "solid-js";
import { Portal } from "solid-js/web";
import { createStore, reconcile } from "solid-js/store";
import { useNavigate } from "@solidjs/router";
import { open } from "@tauri-apps/plugin-dialog"

import { emit, listen, invoke, unlisten, stringToOsString, osStringToString } from "@/logic/backend";
import { Info } from "@/types/data";
import { OsString } from "@/types/rust";
import PluginEntry, { AbortInfo } from "@/components/PluginEntry";
import PluginDialog from "@/components/PluginDialog";
import { setFolders } from "@/logic/folder_state";

async function updateFileTree() {
  return invoke("filetree").then(payload => setFolders(
    mapValues(payload, v =>
      Object.fromEntries(v.map(([filename, isFolder]) =>
        [
          osStringToString(filename),
          {
            folder: isFolder,
            synced: false,
            loading: false
          }
        ]
      ))
    ))
  )
}

export default function PluginSelect() {
  const navigate = useNavigate()

  const [services, setServices] = createStore<Info[]>([]);
  const [loggedInPlugins, setLoggedInPlugins] = createStore<Record<string, boolean>>({})
  const [loading, setLoading] = createSignal<AbortInfo | undefined>()
  const [filepath, setFilepath] = createSignal<string>("")

  function onFileSelected(filepath: string | null) {
    if (!filepath) return
    setFilepath(filepath)
  }

  function init(pair: AbortInfo) {
    setLoading(pair)
    emit("init", pair.filename)
  }

  unlisten([
    listen("init_result", () => {
      updateFileTree().then(() => navigate("/tags"))
    })
  ])()

  // run on app boot
  emit("saved_plugin")

  const refresh = () => invoke("get_plugins").then(plugins => {
    setServices(reconcile(sortBy(plugins, "name")))
    return map(plugins, "filename")
  }).then(plugins =>
    Promise.all(plugins.map(async s => [osStringToString(s), await invoke("logged_in", { filepath: s })] as const))
  ).then(entries => setLoggedInPlugins(Object.fromEntries(entries)));

  onMount(refresh)

  function onClickOpenFileSelector() {
    open({
      directory: false,
      fileAccessMode: "scoped",
      multiple: false,
      filters: [{ extensions: ["dll", "so"], name: "Plugin" }],
      title: "Select plugin file"
    }).then(onFileSelected)
  }

  function addPlugin() {
    invoke("add_plugin", { filepath: stringToOsString(filepath()) }).then(refresh)
    setFilepath("")
  }

  function logout(filepath: OsString) {
    return (e: MouseEvent) => {
      e.preventDefault()
      e.stopPropagation()
      invoke("logout", { filepath }).then(refresh)
    }
  }

  return <>
    <PluginDialog
      filepath={filepath()}
      addPlugin={addPlugin}
      reset={() => setFilepath("")}
    />
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
        <h1>Welcome to SaveSync</h1>
        <div class="space-y-5">
          <Index each={services}>
            {elem =>
              <PluginEntry
                init={() => init(pick(elem(), ["name", "filename"]))}
                elem={elem()}
                loggedIn={() => loggedInPlugins[osStringToString(elem().filename)]}
                logout={logout(elem().filename)}
              />
            }
          </Index>
          <div onclick={onClickOpenFileSelector}
            class="border items-center justify-center p-3 cursor-pointer rounded-lg inline-block"
          >
            <h2>Select a plugin to open</h2>
          </div>
        </div>
      </Show>
    </main >
  </>
}
