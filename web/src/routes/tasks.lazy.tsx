import { createLazyFileRoute } from '@tanstack/react-router'
import { trpc } from "../trpc.ts";
import { TaskList } from '../components/task/TaskList.tsx';
import { useEffect, useState } from 'react';

export const Route = createLazyFileRoute('/tasks')({
  component: Tasks,
})

function Tasks() {
  const [title, setTitle] = useState("");
  const [url, setUrl] = useState("");
  const taskQuery = trpc.tasks.useQuery();
  const newEpisodeMutation = trpc.newEpisode.useMutation();

  useEffect(() => {
    const id = setInterval(() => {
      taskQuery.refetch();
    }, 3000);
    return () => clearInterval(id);
  }, [])

  const onClickCreate = () => {
    newEpisodeMutation.mutate({ title, url })
    setTitle("");
    setUrl("");
    taskQuery.refetch();
  }

  return (
    <div>
      <div>
        <label>title</label>
        <input type="text" value={title} onChange={(e) => setTitle(e.target.value)} />

        <label>url</label>
        <input type="text" value={url} onChange={(e) => setUrl(e.target.value)} />
      </div>
      <button onClick={onClickCreate}>Create Task</button>
      { /* @ts-ignore: TS2589 */}
      <TaskList tasks={taskQuery.data?.tasks ?? []} />
    </div>
  );
}
