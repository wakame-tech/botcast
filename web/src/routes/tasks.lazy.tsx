import { createLazyFileRoute } from '@tanstack/react-router'
import { trpc } from "../trpc.ts";
import { TaskList } from '../components/task/TaskList.tsx';

export const Route = createLazyFileRoute('/tasks')({
  component: Tasks,
})

function Tasks() {
  const taskQuery = trpc.tasks.useQuery();

  return (
    <div>
      <TaskList tasks={taskQuery.data?.tasks ?? []} />
    </div>
  );
}
