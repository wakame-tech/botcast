import { createLazyFileRoute } from '@tanstack/react-router'

export const Route = createLazyFileRoute('/users/$userId')({
  component: User,
})

function User() {
  const { userId } = Route.useParams()

  return (
    <div>
      <h1>{userId}</h1>
    </div>
  );
}
