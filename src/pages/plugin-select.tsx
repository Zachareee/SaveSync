import { createSignal, Index, onMount, Show } from "solid-js";
import { Portal } from "solid-js/web";
import { createStore, reconcile } from "solid-js/store";
import { useNavigate } from "@solidjs/router";
import { open } from "@tauri-apps/plugin-dialog"

import { emit, listen, invoke, unlisten, stringToOsString, osStringToString } from "@/logic/backend";
import { Info } from "@/types/data";
import { Button, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle } from "@suid/material";
import Logout from "@suid/icons-material/Logout";
import { OsString } from "@/types/rust";

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
    listen("init_result", () => navigate("/tags"))
  ])()

  // run on app boot
  emit("saved_plugin")

  const refresh = () => invoke("get_plugins").then(plugins => {
    setServices(reconcile(plugins.sort((p1, p2) => p1.name.localeCompare(p2.name))))
    return plugins.map(({ filename }) => filename)
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

  function logout(plugin: OsString) {
    return (e: MouseEvent) => {
      e.stopPropagation()
      invoke("logout", { filepath: plugin }).then(refresh)
    }
  }

  return <>
    <Dialog open={filepath()?.length != 0}>
      <DialogTitle>Are you sure you want to add this plugin?</DialogTitle>
      <DialogContent>
        <DialogContentText>
          Plugins may be created with malicious intent which may compromise your computer. Only add a plugin if you trust the author of the plugin and its contents
        </DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={addPlugin}>Add</Button>
        <Button onClick={[setFilepath, ""]}>Cancel</Button>
      </DialogActions>
    </Dialog>
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
                  <img src={elem().icon_url} class="w-10 mr-4 inline-block content-center" />
                </Show>
                <div class="inline-block">
                  <h2>{elem().name}</h2>
                  <p>Description: {elem().description}</p>
                  <span>Written by: {elem().author}</span>
                </div>
                <div class="inline-block">
                  <Show when={loggedInPlugins[osStringToString(elem().filename)]}>
                    <Logout class="bg-red-600 size-full content-center hover:outline-2" onClick={logout(elem().filename)} />
                  </Show>
                </div>
              </div>
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

type AbortInfo = Pick<Info, "name" | "filename">
