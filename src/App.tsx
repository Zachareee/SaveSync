import "./App.css";
import { onCleanup, onMount } from "solid-js";
import { Route, Router } from "@solidjs/router";
import { Toaster } from "solid-toast";

import { getCurrentWindow } from "@tauri-apps/api/window";
import { exit } from "@tauri-apps/plugin-process"

import { createWindow } from "@/logic/window";
import { listen, osStringToString } from "@/logic/backend";
import { setFolders } from "@/logic/folder_state";
import { Conflicting, ErrorPage, Folders, Mapping, PluginSelect, Tags, Settings, Profiles } from "@/pages";

(() => {
  if (getCurrentWindow().label === "main")
    listen("plugin_error", ([title, error]) => createWindow(`/error?${new URLSearchParams({ error })}`, { title: osStringToString(title), parent: "main" }))
})();

listen("sync_result", ([tag, folder, bool]) => {
  setFolders(tag, osStringToString(folder), { loading: false, synced: bool })
});

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
      <Route path={"/tags"} component={Tags("tags")} />
      <Route path={"/tags/:TAGNAME"} component={Folders} />
      <Route path={"/profiles"} component={Tags("profiles")} />
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
