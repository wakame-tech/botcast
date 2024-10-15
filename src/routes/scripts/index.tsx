import { trpc } from "@/trpc.ts";
import { Link, createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/scripts/')({
  component: Scripts,
})

export function Scripts() {
  const getScripts = trpc.scripts.useQuery()

  if (!getScripts.data) {
    return null
  }

  const scripts = getScripts.data.scripts

  return (
    <>
      <Link to="/scripts/new">New Script</Link>

      <ul>
        {/* @ts-ignore */}
        {scripts.map(script => (
          <li key={script.id}>
            <Link to="/scripts/$scriptId" params={{ scriptId: script.id }}>
              {script.title}
            </Link>
          </li>
        ))}
      </ul>
    </>
  )
}
