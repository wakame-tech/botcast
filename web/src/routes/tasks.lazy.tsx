import { createLazyFileRoute } from '@tanstack/react-router'
import { trpc } from "../trpc.ts";

export const Route = createLazyFileRoute('/tasks')({
  component: Tasks,
})

function Tasks() {
  const taskQuery = trpc.tasks.useQuery();

  return (
    <div>
      {JSON.stringify(taskQuery.data?.tasks)}
    </div>
  );
}
