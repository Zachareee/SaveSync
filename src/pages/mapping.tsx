import { invoke, osStringToString, stringToOsString } from "@/logic/all-backend"
import { EnvMapping, FolderMapping } from "@/types"
import { Index, Show } from "solid-js"
import { createStore } from "solid-js/store"
import { Portal } from "solid-js/web"
import { open } from "@tauri-apps/plugin-dialog"

type MappingArray = ReturnType<typeof Object.entries<FolderMapping[string]>>

const createAddPath = (setMapping: ReturnType<typeof createStore<MappingArray>>[1]) =>
  () => setMapping(mapping => [...mapping, ["", ["", { Windows: [] }]]])
const createRemovePath = (setMapping: ReturnType<typeof createStore<MappingArray>>[1]) =>
  (idx: number) => setMapping(mapping => mapping.toSpliced(idx, 1))

export default function Mapping() {
  const [envs, setEnvs] = createStore<EnvMapping>()
  const [mapping, setMapping] = createStore<MappingArray>([])
  invoke("get_mapping").then(m => setMapping(Object.entries(m)))
  invoke("get_envpaths").then(setEnvs)

  const addPath = createAddPath(setMapping)
  const removePath = createRemovePath(setMapping)

  return <>
    <div>
      <Index each={mapping}>
        {(elem, idx) => <div class="flex">
          <input value={elem()[0]} onchange={e => setMapping(idx, 0, e.target.value)} />
          <div>
            <div>
              <select id={`${idx}`}
                value={elem()[1][0]}
                onchange={e => setMapping(idx, 1, 0, e.target.value)}
                class="border-white border-2 rounded-lg"
              >
                <Index each={Object.entries(envs).sort()}>
                  {
                    option => <option class="bg-black"> {option()[0]} </option>
                  }
                </Index>
              </select>
              <input value={osStringToString(elem()[1][1])} disabled />
              <button
                onclick={() => open({ directory: true, multiple: false, defaultPath: osStringToString(envs[elem()[1][0]]) }).then(path => {
                  if (path)
                    setMapping(idx, 1, 1, stringToOsString(path.replace(`${osStringToString(envs[elem()[1][0]])}\\`, "")))
                })}
              >Browse</button>
            </div>
            <span>Path: {`${osStringToString(envs[elem()[1][0]])}\\${osStringToString(elem()[1][1])}`}</span>
          </div>
          <button onclick={[removePath, idx]}>Delete mapping</button>
        </div>}
      </Index>
      <Portal>
        <div class="absolute right-0 bottom-0 m-4">
          <button onclick={addPath}>Add mapping</button>
        </div>
      </Portal>
    </div>
  </>
}
