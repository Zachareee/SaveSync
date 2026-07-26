import { Navigator } from "@solidjs/router";
import { Component, For } from "solid-js";
import NavOption from "./NavOption";

const navs = Object.entries({
  Folders: "/folders",
  "Tag mapping": "/mapping"
})

const NavBar: Component<{ navigate: Navigator }> = props => {
  return <>
    <div class="inline-block h-screen content-center bg-indigo-950 p-2">
      <For each={navs}>
        {name => <NavOption onclick={[props.navigate, name[1]]}>{name[0]}</NavOption>}
      </For>
    </div>
  </>
}

export default NavBar
