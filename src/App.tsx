import "./App.css";
import { onCleanup, onMount } from "solid-js";
import { createStore } from "solid-js/store";
import { Route, Router } from "@solidjs/router";
import { Toaster } from "solid-toast";

import { getCurrentWindow } from "@tauri-apps/api/window";
import { exit } from "@tauri-apps/plugin-process"

import { createWindow } from "@/logic/window";
import { listen, osStringToString } from "@/logic/backend";
import { FileTree } from "@/types/data";
import { Conflicting, ErrorPage, Folders, Mapping, PluginSelect, Tags, Settings } from "@/pages";

export const [folders, setFolders] = createStore<FileTree>();

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
      <Route path={"/tags"} component={Tags} />
      <Route path={"/tags/:TAGNAME"} component={Folders} />
      <Route path={"/error"} component={ErrorPage} />
      <Route path={"/mapping"} component={Mapping} />
      <Route path={"/conflicting"} component={Conflicting} />
      <Route path={"/settings"} component={Settings} />
      <Route path={"*"} component={PluginSelect} />
    </Router>
  </>
}

export default App;
