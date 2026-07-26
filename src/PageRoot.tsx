import { useNavigate } from "@solidjs/router"
import { createSignal, ParentComponent, Show } from "solid-js"
import MenuButton from "./components/MenuButton"
import NavBar from "./components/NavBar"

const PageRoot: ParentComponent = props => {
  const [showMenu, setShowMenu] = createSignal(false)
  const navigate = useNavigate()

  return <>
    <Show when={showMenu()}>
      <NavBar navigate={navigate} />
    </Show>
    <MenuButton onclick={[setShowMenu, (m: boolean) => !m]} class="fixed" />
    <div class="inline-block w-full h-screen">
      {props.children}
    </div>
  </>
}

export default PageRoot
