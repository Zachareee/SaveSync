import { invoke, osStringToString } from "@/logic/backend"
import { FolderMapping } from "@/types"
import { Index } from "solid-js"
import { createStore } from "solid-js/store"

export default function Mapping() {
  const [mapping, setMapping] = createStore<FolderMapping>({})
  invoke("get_mapping").then(setMapping)

  return <div>
    <Index each={Object.entries(mapping)}>
      {elem => <div class="flex">
        <span>{elem()[0]}</span>
        <p>{osStringToString(elem()[1])}</p>
      </div>
      }
    </Index>
  </div>
}
