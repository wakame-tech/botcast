import { SupabaseClient } from "../../deps.ts";
import { Feed } from "../../model.ts";
import { IFeedRepository } from "./index.ts";

export class SupabaseFeedRepository implements IFeedRepository {
  constructor(private supabase: SupabaseClient) {}

  async getAll(): Promise<Feed[]> {
    const { data, error } = await this.supabase.from("feeds");
    if (error) {
      throw error;
    }
    return data;
  }

  async create(feed: Feed): Promise<void> {
    const res = await this.supabase.from("feeds").upsert(feed);
    if (res.error) {
      throw res.error;
    }
  }
}
