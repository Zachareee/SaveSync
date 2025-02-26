import { Menu, MenuItem, Submenu } from "@tauri-apps/api/menu";
import { createWindow } from "./window";
import { Window } from "@tauri-apps/api/window";

export const mainMenu = await (async () => {
  if (Window.getCurrent().label == "main") {
    const menu = await Menu.new({
      items: await Promise.all([
        Submenu.new({
          text: "File", id: "file", items: [
            {
              text: "Mapping",
              id: "mapping",
              enabled: false,
              action() {
                createWindow("/mapping", { title: "Mapping", parent: "main" })
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
  return mainMenu!.get("file").then(m => (m as Submenu).get("mapping")).then(i => (i as MenuItem).setEnabled(active))
}
