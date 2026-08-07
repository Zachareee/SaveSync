import { Component, JSX } from "solid-js";

const NavOption: Component<JSX.HTMLAttributes<HTMLDivElement> & { active: boolean }> = props => {
  return <>
    <div onclick={props.onclick} class="text-center p-2 hover:bg-indigo-800 cursor-pointer min-w-max" classList={{ "bg-indigo-800": props.active }}>
      <h1 class="font-bold text-xl">{props.children}</h1>
    </div>
  </>
}

export default NavOption
