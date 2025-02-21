import { invoke } from "@/logic/all-backend"
import { EnvMapping, FolderMapping } from "@/types"
import { Index } from "solid-js"
import { createStore } from "solid-js/store"
import { Portal } from "solid-js/web"

type MappingArray = ReturnType<typeof Object.entries<FolderMapping[string]>>

const createAddPath = (setMapping: ReturnType<typeof createStore<MappingArray>>[1]) =>
  () => setMapping(mapping => [...mapping, ["hi", ["APPDATA", { Windows: [] }]]])
const createRemovePath = (setMapping: ReturnType<typeof createStore<MappingArray>>[1]) =>
  (idx: number) => setMapping(mapping => { console.table(JSON.stringify(mapping)); mapping = mapping.toSpliced(idx, 1); return mapping })

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
          <span>{elem()[0]}{idx}</span>
          <div>
            <div>
              <select id={`${idx}`}
                value={elem()[1][0]}
                onchange={e => setMapping(idx, 1, 0, e.target.value)}
              >
                <Index each={Object.entries(envs).sort()}>
                  {
                    option => <option> {option()[0]} </option>
                  }
                </Index>
              </select>
            </div>
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
