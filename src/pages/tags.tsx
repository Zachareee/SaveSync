import { useNavigate } from "@solidjs/router"
import { createSignal, For } from "solid-js"
import toast from "solid-toast"
import { includes, negate, partial } from "lodash"
import { Portal } from "solid-js/web"
import { load } from "@tauri-apps/plugin-store"

import { folders } from "@/logic/folder_state"
import PageRoot from "@/PageRoot"
import { emit, invoke, unlisten } from "@/logic/backend"
import { conflicting_listener } from "@/logic/conflicting_window"

export default function Tags(path: string) {
  function _Tags() {
    const navigate = useNavigate()
    const [silenceMissingMappings, setSilenceMappingsMissing] = createSignal<boolean>(false)

    const store = load("store.json")
    store.then(s =>
      s.get<boolean>("silenceMissingMappings").then(setSilenceMappingsMissing)
    ).then(() =>
      invoke("get_mapping").then(({ mapping, required }) => {
        const current = Object.keys(mapping)
        if (required.some(negate(partial(includes, current))) && !silenceMissingMappings())
          toast.error(
            (t) =>
              <p onclick={() => {
                toast.dismiss(t.id)
                navigate("/mapping")
              }}>Some folders were not synced, please check Tag Mapping</p>
          )
      })
    )

    unlisten([conflicting_listener()])()

    return <PageRoot>
      <div class="flex justify-center overflow-y-auto">
        <div class="text-center">
          <For each={Object.keys(folders).toSorted()}>
            {elem =>
              <div class="border-white border-3 rounded-full m-4 px-4 py-2 bg-indigo-600 cursor-pointer hover:bg-indigo-800" onclick={[navigate, `/${path}/${encodeURIComponent(elem)}`]}>
                {elem}
              </div>
            }
          </For>
        </div>
      </div>
    </PageRoot>
  }
  return _Tags
}

