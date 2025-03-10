import { Menu, MenuItem, Submenu } from "@tauri-apps/api/menu";
import { createWindow } from "./window";
import { Window } from "@tauri-apps/api/window";

export const mainMenu = await (async () => {
  const parent = Window.getCurrent()
  if (parent.label == "main") {
    const menu = await Menu.new({
      items: await Promise.all([
        Submenu.new({
          text: "File", id: "file", items: [
            {
              text: "Mapping",
              id: "mapping",
              enabled: false,
              action() {
                createWindow("/mapping", { title: "Mapping", parent })
              }
            }
          ]
        })
      ])
    })
    menu.setAsWindowMenu()
    return menu
  }
})()

export async function menuStatus(active: boolean) {
  Object.entries(toggleMenuOptions).forEach(
    ([key, options]) => mainMenu!.get(key).then(
      m => options.forEach(
        o => (m as Submenu).get(o).then(i => (i as MenuItem).setEnabled(active))
      )
    )
  )
}

const toggleMenuOptions: Record<string, string[]> = {
  file: ["mapping"]
}
