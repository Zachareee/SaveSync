import { useNavigate } from "@solidjs/router"
import { createSignal, ParentComponent, Show } from "solid-js"
import MenuButton from "./components/MenuButton"
import NavBar from "./components/NavBar"

const PageRoot: ParentComponent = props => {
  const [showMenu, setShowMenu] = createSignal(false)
  const navigate = useNavigate()

  return <>
    <MenuButton onclick={[setShowMenu, (m: boolean) => !m]} class="absolute"/>
    <div>
      <Show when={showMenu()}>
          <NavBar navigate={navigate}/>
      </Show>
      <div class="inline-block">
        {props.children}
      </div>
    </div>
  </>
}

export default PageRoot
