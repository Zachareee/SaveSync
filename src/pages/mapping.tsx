import { invoke, osStringToString, stringToOsString } from "@/logic/backend"
import { For, Index } from "solid-js"
import { createStore } from "solid-js/store"
import { Portal } from "solid-js/web"
import { useNavigate } from "@solidjs/router"
import { open } from "@tauri-apps/plugin-dialog"
import { RequiredList } from "@/types/data"
import PageRoot from "@/PageRoot"

type MappingArray = [string, string][]

const createAddPath = (setMapping: ReturnType<typeof createStore<MappingArray>>[1]) =>
  (tag: string) => setMapping(mapping => [...mapping, [tag, ""] as const])
const createRemovePath = (setMapping: ReturnType<typeof createStore<MappingArray>>[1]) =>
  (idx: number) => setMapping(mapping => mapping.toSpliced(idx, 1))

function saveAndClose([mapping, navigate]: [MappingArray, () => {}]) {
  invoke("set_mapping", {
    map: Object.fromEntries(
      mapping.filter(validEntry).map(e => [e[0], stringToOsString(e[1])])
    )
  }).then(navigate)
}

function validEntry(entry: MappingArray[number]): boolean {
  return entry.every(e => e)
}

export default function Mapping() {
  const [mapping, setMapping] = createStore<MappingArray>([])
  const [requiredList, setRequiredList] = createStore<RequiredList>([])

  invoke("get_mapping").then(({ mapping, required }) => {
    setMapping(Object.entries(mapping).map(e => [e[0], osStringToString(e[1])] as [string, string]).toSorted(([a, _a], [b, _b]) => a.localeCompare(b)))
    setRequiredList(required)
  })

  const addPath = createAddPath(setMapping)
  const removePath = createRemovePath(setMapping)

  const navigate = useNavigate()

  return <PageRoot>
    <div class="h-screen flex-col content-center">
      <div class="flex flex-col items-center overflow-y-auto max-h-1/2">
        <div class="space-y-2">
          <Index each={mapping}>
            {(elem, idx) =>
              <div class="text-center space-x-0.5">
                <input value={elem()[0]} onInput={e => setMapping(idx, 0, e.target.value)} class="w-min" />
                <input value={elem()[1]} disabled />
                <button
                  onclick={() => open({ directory: true, multiple: false }).then(path => {
                    if (path)
                      setMapping(idx, 1, path)
                  })}
                >Browse</button>
                <button onclick={[removePath, idx]}>Delete</button>
              </div>
            }
          </Index>
        </div>
      </div>
      <br />
      <div class="flex-col text-center">
        <h2> Missing tags </h2>
        <For each={requiredList.filter(tag => !mapping.some(([key]) => tag == key))}>
          {e => <button onclick={[addPath, e]}>{e}</button>}
        </For>
      </div>
      <div class="flex justify-between self-end">
        <div class="m-4">
          <button onclick={[addPath, ""]}>Add mapping</button>
        </div>
        <div class="m-4">
          <button onclick={[saveAndClose, [mapping, () => navigate("/folders")]]}>Save and close</button>
        </div>
      </div>
    </div>
  </PageRoot>
}
