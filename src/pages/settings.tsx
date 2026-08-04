import PageRoot from "@/PageRoot"
import Switch from "@suid/material/Switch"
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart"
import { load } from '@tauri-apps/plugin-store'
import { createSignal } from "solid-js"

export default function Settings() {
  const [autoStartup, setAutoStartup] = createSignal<boolean>(false)
  const [closeBehaviour, setCloseBehaviour] = createSignal<boolean>(true)
  const [silenceMissingMappings, setSilenceMappingsMissing] = createSignal<boolean>(false)

  isEnabled().then(setAutoStartup)

  const store = load("store.json")
  store.then(s => {
    s.get<boolean>("silenceMissingMappings").then(setSilenceMappingsMissing)
    s.get<boolean>("hide_to_tray").then(setCloseBehaviour)
  })

  async function toggleAutoStartup(_: any, checked: boolean) {
    await (checked ? enable : disable)()
    setAutoStartup(checked)
  }

  function toggleSetterAndSave(setter: ReturnType<typeof createSignal<boolean>>[1], key: string) {
    return async (_: any, checked: boolean) => {
      setter(checked)
      await store.then(async s => {
        await s.set(key, checked)
        await s.save()
      })
    }
  }

  return <PageRoot>
    <div class="overflow-y-scroll m-4">
      <h1 class="font-bold text-3xl justify-self-start mb-2">Notifications</h1>
      <div class="border-2 border-indigo-200 rounded-xl p-2">
        <span>Silence "Mappings missing" notifications</span>
        <Switch onChange={toggleSetterAndSave(setSilenceMappingsMissing, "silenceMissingMappings")} checked={silenceMissingMappings()} />
      </div>
      <br />
      <h1 class="font-bold text-3xl justify-self-start mb-2">Behaviour</h1>
      <div class="border-2 border-indigo-200 rounded-xl p-2">
        <div>
          <span>Startup when computer powers on</span>
          <Switch onChange={toggleAutoStartup} checked={autoStartup()} />
        </div>
        <div>
          <span>Hide when closed</span>
          <Switch onChange={toggleSetterAndSave(setCloseBehaviour, "hide_to_tray")} checked={closeBehaviour()} />
        </div>
      </div>
    </div>
  </PageRoot>
}
