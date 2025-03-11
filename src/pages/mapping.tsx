import { emit, invoke, osStringToString, stringToOsString } from "@/logic/backend"
import { For, Index, Show } from "solid-js"
import { createStore } from "solid-js/store"
import { Portal } from "solid-js/web"
import { confirm, open } from "@tauri-apps/plugin-dialog"
import { WebviewWindow } from "@tauri-apps/api/webviewWindow"
import lo from "lodash"
import { RequiredList } from "@/types"

type MappingArray = [string, [string, string]][]

const createAddPath = (setMapping: ReturnType<typeof createStore<MappingArray>>[1]) =>
  (tag: string) => setMapping(mapping => [...mapping, [tag, ["", ""]] as const])
const createRemovePath = (setMapping: ReturnType<typeof createStore<MappingArray>>[1]) =>
  (idx: number) => setMapping(mapping => mapping.toSpliced(idx, 1))

const window = WebviewWindow.getCurrent()
function saveAndClose(mapping: MappingArray) {
  invoke("set_mapping", {
    map: Object.fromEntries(
      mapping.filter(validEntry).map(e => [e[0], [e[1][0], stringToOsString(e[1][1])]])
    )
  }).then(() => {
    emit("filetree")
    window.destroy()
  })
}

function validEntry(entry: MappingArray[number]): string {
  return entry[0] && entry[1][0]
}

export default function Mapping() {
  const [envs, setEnvs] = createStore<Record<string, string>>()
  const [mapping, setMapping] = createStore<MappingArray>([])
  const [oMapping, setOMapping] = createStore<MappingArray>([])
  const [requiredList, setRequiredList] = createStore<RequiredList>([])

  invoke("get_envpaths").then(e => setEnvs(Object.fromEntries(Object.entries(e).map(([name, path]) => [name, osStringToString(path)]))))
  invoke("get_mapping").then(({ mapping, required }) => {
    [setMapping, setOMapping].forEach(f => f(Object.entries(mapping).map(e => [e[0], [e[1][0], osStringToString(e[1][1])]])))
    setRequiredList(required)
  })

  const addPath = createAddPath(setMapping)
  const removePath = createRemovePath(setMapping)

  window.onCloseRequested(async e => {
    if (lo.isEqual(mapping.filter(validEntry), oMapping) || await confirm("Unsaved changes will be lost"))
      return window.destroy()
    e.preventDefault()
  })

  return <>
    <div>
      <Index each={mapping}>
        {(elem, idx) => <div class="flex">
          <input value={elem()[0]} onInput={e => setMapping(idx, 0, e.target.value)} />
          <div>
            <div>
              <select id={`${idx}`}
                value={elem()[1][0]}
                onchange={e => setMapping(idx, 1, 0, e.currentTarget.value)}
                class="border-white border-2 rounded-lg"
              >
                <option class="bg-black" />
                <Index each={Object.entries(envs).sort()}>
                  {
                    option => <option class="bg-black"> {option()[0]} </option>
                  }
                </Index>
              </select>
              <input value={elem()[1][1]} disabled />
              <button
                onclick={() => open({ directory: true, multiple: false, defaultPath: envs[elem()[1][0]] }).then(path => {
                  if (path)
                    setMapping(idx, 1, 1, path.replace(RegExp(`${envs[elem()[1][0]]}\\?`.replace(/\\/g, "\\\\"), "g"), ""))
                })}
              >Browse</button>
            </div>
            <Show when={envs[elem()[1][0]]}>
              <span>Path: {`${envs[elem()[1][0]]}\\${elem()[1][1]}`}</span>
            </Show>
          </div>
          <button onclick={[removePath, idx]}>Delete mapping</button>
        </div>}
      </Index>
      <h2> Missing tags </h2>
      <For each={requiredList.filter(tag => !mapping.some(([key]) => tag == key))}>
        {e => <button onclick={[addPath, e]}>{e}</button>}
      </For>
      <Portal>
        <div class="fixed left-0 bottom-0 m-4">
          <button onclick={[addPath, ""]}>Add mapping</button>
        </div>
        <div class="fixed right-0 bottom-0 m-4">
          <button onclick={[saveAndClose, mapping]}>Save and quit</button>
        </div>
      </Portal>
    </div>
  </>
}
