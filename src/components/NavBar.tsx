import { Navigator, useLocation } from "@solidjs/router";
import { Component, For } from "solid-js";
import NavOption from "./NavOption";
import { emit } from "@/logic/backend";

const navs = Object.entries({
  "Folders": "/tags",
  "Tag mapping": "/mapping",
  "Save profiles": "/profiles",
  "Settings": "/settings"
})

const NavBar: Component<{ navigate: Navigator }> = ({ navigate }) => {
  const location = useLocation().pathname
  return <>
    <div class="content-center bg-indigo-950 p-2">
      <div>
        <For each={navs}>
          {name => <NavOption onclick={[navigate, name[1]]} active={location.startsWith(name[1])}>{name[0]}</NavOption>}
        </For>
      </div>
      <div class="self-end">
        <NavOption onclick={() => { emit("unload"); navigate("/") }}>Plugins</NavOption>
      </div>
    </div>
  </>
}

export default NavBar
