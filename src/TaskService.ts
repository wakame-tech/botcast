import { FeedService, feedService } from "./FeedService.ts";
import { ScriptService, scriptService } from "./ScriptService.ts";
import { supabase } from "./lib/supabase.ts";
import { GenerateFeedTask, Source } from "./model.ts";
import { SupabaseGenerateTaskRepository } from "./repo/feed/GenerateTaskRepository.ts";

export class TaskService {
  constructor(
    private scriptService: ScriptService,
    private feedService: FeedService,
    private taskRepo: SupabaseGenerateTaskRepository
  ) {}

  async push(title: string, sources: Source[]): Promise<GenerateFeedTask> {
    const task: GenerateFeedTask = {
      id: crypto.randomUUID(),
      title,
      status: "queued",
      sources,
      date: new Date().toISOString(),
    };
    await this.taskRepo.push(task);
    return task;
  }

  poll() {
    setInterval(async () => {
      const task = await this.taskRepo.pop();
      if (!task) {
        return;
      }
      console.log(
        `[tasks] ${task.id} ${task.title} ${task.sources.length} sources}`
      );
      try {
        console.log(`${task.id} -> processing`);
        await this.taskRepo.update(task.id, "processing");

        const script = await this.scriptService.generate(
          task.title,
          task.sources
        );
        const audio = await this.scriptService.synthesis(script);
        await this.feedService.post(script, audio);
        console.log(`${task.id} -> success`);
        await this.taskRepo.update(task.id, "success");
      } catch (e) {
        console.log(e);
        console.log(`${task.id} -> failed`);
        await this.taskRepo.update(task.id, "failed");
      }
    }, 5 * 1000);
  }
}

export const taskService = new TaskService(
  scriptService,
  feedService,
  new SupabaseGenerateTaskRepository(supabase)
);
