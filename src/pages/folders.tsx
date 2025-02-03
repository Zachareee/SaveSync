import { useNavigate } from "@solidjs/router"

export default function Folders() {
  const navigate = useNavigate()
  return <div>
    <button onclick={() => navigate("/")}>Back to plugin select</button>
  </div>
}
