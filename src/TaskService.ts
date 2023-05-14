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
      console.log(`[tasks] ${task}`);
      if (!task) {
        return;
      }
      try {
        console.log(`${task.id} -> processing`);
        await this.taskRepo.update(task.id, "processing");

        const script = await this.scriptService.generate(
          task.title,
          task.sources
        );
        console.log(script);
        const audio = await this.scriptService.synthesis(script);
        await this.feedService.post(script, audio);
        await this.taskRepo.update(task.id, "success");
      } catch (e) {
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
