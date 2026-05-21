import { Component, JSX } from "solid-js";

const NavOption: Component<JSX.HTMLAttributes<HTMLDivElement>> = props => {
  return <>
    <div onclick={props.onclick} class="text-center border-2 p-1">
      {props.children}
    </div>
  </>
}

export default NavOption
