import "./App.css";
import { onCleanup, onMount } from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import { Route, Router } from "@solidjs/router";
import { Toaster } from "solid-toast";
import { isEqual, mapValues, partialRight, unary } from "lodash";

import { getCurrentWindow } from "@tauri-apps/api/window";
import { exit } from "@tauri-apps/plugin-process"

import { createWindow } from "@/logic/window";
import { listen, invoke, osStringToString } from "@/logic/backend";
import { FileTree } from "@/types/data";
import { Conflicting, ErrorPage, Folders, Mapping, PluginSelect, Tags, Settings, Profiles } from "@/pages";

export const [folders, setFolders] = createStore<FileTree>();
export function updateFileTree() {
  invoke("filetree").then(payload => {
    invoke("get_watched_folders").then(watched => {
      setFolders(reconcile(
        mapValues(payload, (v, k) =>
          Object.fromEntries(v.map(([filename, isFolder]) =>
            [
              osStringToString(filename),
              {
                folder: isFolder,
                synced: watched.some(
                  unary(partialRight(isEqual, [k, filename]))
                ),
                loading: false
              }
            ]
          ))
        )
      ))
    })
  })
}

(() => {
  if (getCurrentWindow().label === "main")
    listen("plugin_error", ([title, error]) => createWindow(`/error?${new URLSearchParams({ error })}`, { title: osStringToString(title), parent: "main" }))
})()

function App() {
  onMount(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      const ctrlPressed = e.ctrlKey || e.metaKey
      const qPressed = e.key.toLowerCase() === "q"

      if (ctrlPressed && qPressed) {
        e.preventDefault()
        e.stopPropagation()

        await exit(0)
      }
    }
    window.addEventListener("keydown", handleKeyDown)

    onCleanup(() => window.removeEventListener("keydown", handleKeyDown))
  })

  return <>
    <Toaster position="bottom-center" containerClassName="cursor-pointer" />
    <Router>
      <Route path={"/tags"} component={Tags("/path")} />
      <Route path={"/tags/:TAGNAME"} component={Folders} />
      <Route path={"/profiles"} component={Tags("/profiles")} />
      <Route path={"/profiles/:TAGNAME"} component={Profiles} />
      <Route path={"/error"} component={ErrorPage} />
      <Route path={"/mapping"} component={Mapping} />
      <Route path={"/conflicting"} component={Conflicting} />
      <Route path={"/settings"} component={Settings} />
      <Route path={"*"} component={PluginSelect} />
    </Router>
  </>
}

export default App;
