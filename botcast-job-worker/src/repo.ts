import { Task, TaskRepo, TaskStatus } from "./worker.ts";
import { Client } from "https://deno.land/x/postgres@v0.17.0/mod.ts";

export type BotcastTask = {
  type: "generate";
  url: string;
};

const from = (entity: any): Task<BotcastTask> => {
  return {
    id: entity.id,
    arg: JSON.parse(entity.body) as BotcastTask,
    status: entity.status as TaskStatus,
    date: entity.date,
  };
};

export class BotcastSynthesisTaskRepo implements TaskRepo<BotcastTask> {
  private table = "Task";

  constructor(private client: Client) {}

  async fetchTasks(): Promise<Task<BotcastTask>[]> {
    const result = await this.client.queryObject(
      `SELECT id,status,body,date from ${this.table}`
    );
    return result.rows.map(from);
  }
  async save(task: Task<BotcastTask>): Promise<Task<BotcastTask>> {
    await this.client.queryArray({
      args: {
        id: task.id,
        status: task.status,
        body: JSON.stringify(task.arg),
        date: task.date,
      },
      text: `INSERT INTO ${this.table} VALUES ($ID,$STATUS,$BODY,$DATE)`,
    });
    return task;
  }
  async fetchNextTask(): Promise<Task<BotcastTask> | null> {
    const result = await this.client.queryObject(
      `SELECT id,status,body,date FROM ${this.table} WHERE status = 'queued' ORDER BY date DESC LIMIT 1`
    );
    return result.rows.map(from)[0];
  }
  async updateStatus(taskId: string, status: TaskStatus): Promise<void> {
    await this.client.queryArray({
      args: {
        id: taskId,
        status,
      },
      text: `UPDATE ${this.table} SET status = $STATUS WHERE id = $ID`,
    });
  }
}

const client = new Client(Deno.env.get("DATABASE_URL"));
export const repo = new BotcastSynthesisTaskRepo(client);
