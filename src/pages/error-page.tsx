import { useParams } from "@solidjs/router"

export default function ErrorPage() {
  const { ERROR } = useParams()

  return <div>
    <p>Error found: {decodeURIComponent(ERROR)}</p>
  </div>
}
