import { createLazyFileRoute } from '@tanstack/react-router'
import { trpc } from '../../trpc';

export const Route = createLazyFileRoute('/users/$userId')({
  component: User,
})

function User() {
  const meQuery = trpc.me.useQuery();
  if (meQuery.error) {
    return <div>Error: {meQuery.error.message}</div>
  }
  if (meQuery.isLoading || !meQuery.data) {
    return <div>Loading...</div>
  }
  const user = meQuery.data.user

  return (
    <div>
      <p>name: {user.name}</p>
      <p>email: {user.email}</p>
    </div>
  );
}
