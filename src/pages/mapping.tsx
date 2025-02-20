import { invoke, osStringToString } from "@/logic/all-backend"
import { FolderMapping } from "@/types"
import { Index } from "solid-js"
import { createStore } from "solid-js/store"
import { Portal } from "solid-js/web"

type MappingArray = ReturnType<typeof Object.entries<FolderMapping[string]>>

const createAddPath = (setMapping: ReturnType<typeof createStore<MappingArray>>[1]) =>
  () => setMapping(mapping => [...mapping, ["hi", { Windows: [] }]])
const createRemovePath = (setMapping: ReturnType<typeof createStore<MappingArray>>[1]) =>
  (idx: number) => setMapping(mapping => mapping.toSpliced(idx, 1))

export default function Mapping() {
  const [mapping, setMapping] = createStore<MappingArray>([])
  invoke("get_mapping").then(m => setMapping(Object.entries(m)))

  const addPath = createAddPath(setMapping)
  const removePath = createRemovePath(setMapping)

  return <>
    <div>
      <Index each={mapping}>
        {(elem, idx) => <div class="flex">
          <span>{elem()[0]}{idx}</span>
          <p>{osStringToString(elem()[1])}</p>
          {
            // please fix, input doesn't move with index
            /*<input onkeydown={e => setMapping(mapping => mapping.with(idx, mapping[idx].with(1, e.currentTarget.value) as [string, string]))} />*/
          }
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
