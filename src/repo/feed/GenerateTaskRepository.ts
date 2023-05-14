import { SupabaseClient } from "../../deps.ts";
import { Feed, GenerateFeedTask } from "../../model.ts";

export class SupabaseGenerateTaskRepository {
  constructor(private supabase: SupabaseClient) {}

  async push(task: GenerateFeedTask): Promise<void> {
    const { error } = await this.supabase
      .from<GenerateFeedTask>("tasks")
      .upsert(task);
    if (error) {
      throw error;
    }
  }

  async update(id: string, status: GenerateFeedTask["status"]): Promise<void> {
    const { error } = await this.supabase
      .from<GenerateFeedTask>("tasks")
      .update({
        status,
      })
      .filter("id", "eq", id);
    if (error) {
      throw error;
    }
  }

  async pop(): Promise<GenerateFeedTask | null> {
    const { data, error } = await this.supabase
      .from<GenerateFeedTask>("tasks")
      .select("*")
      .filter("status", "eq", "queued")
      .order("date", { ascending: true })
      .limit(1);
    if (error) {
      throw error;
    }
    const row = data[0];
    if (!row) {
      return null;
    }
    return GenerateFeedTask.parse(row);
  }
}
