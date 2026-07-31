import { Navigator, useLocation } from "@solidjs/router";
import { Component, For } from "solid-js";
import NavOption from "./NavOption";

const navs = Object.entries({
  "Folders": "/tags",
  "Tag mapping": "/mapping",
  "Settings": "/settings"
})

const NavBar: Component<{ navigate: Navigator }> = props => {
  const location = useLocation().pathname
  return <>
    <div class="inline-block h-screen content-center bg-indigo-950 p-2">
      <For each={navs}>
        {name => <NavOption onclick={[props.navigate, name[1]]} active={location.startsWith(name[1])}>{name[0]}</NavOption>}
      </For>
    </div>
  </>
}

export default NavBar
