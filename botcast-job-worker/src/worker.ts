export type TaskStatus = "queued" | "processing" | "success" | "failed";

export interface Task<T> {
  id: string;
  status: TaskStatus;
  arg: T;
  date: string;
}

export interface TaskRepo<T> {
  fetchTasks(): Promise<Task<T>[]>;
  save(task: Task<T>): Promise<Task<T>>;
  fetchNextTask(): Promise<Task<T> | null>;
  updateStatus(taskId: string, status: TaskStatus): Promise<void>;
}

export const pushTask = <T>(repo: TaskRepo<T>, arg: T): Promise<Task<T>> => {
  const task = {
    id: crypto.randomUUID(),
    status: "queued" as TaskStatus,
    date: new Date().toISOString(),
    arg,
  };
  return repo.save(task);
};

export const poll = <T>(
  repo: TaskRepo<T>,
  callback: (arg: T) => Promise<void>,
  interval = 5000
) => {
  setInterval(async () => {
    console.log(`[worker] polling`);
    const task = await repo.fetchNextTask();
    if (!task) {
      return;
    }
    try {
      console.log(`${task.id} -> processing`);
      await repo.updateStatus(task.id, "processing");
      await callback(task.arg);
      console.log(`${task.id} -> success`);
      await repo.updateStatus(task.id, "success");
    } catch (e) {
      console.log(e);
      console.log(`${task.id} -> failed`);
      await repo.updateStatus(task.id, "failed");
    }
  }, interval);
};
