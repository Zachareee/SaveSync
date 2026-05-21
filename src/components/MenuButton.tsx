import { Component, JSX } from "solid-js";

const MenuButton: Component<JSX.ButtonHTMLAttributes<HTMLButtonElement>> = props => {
  return <>
    <button onclick={props.onclick} class={props.class}>☰</button>
  </>
}

export default MenuButton
