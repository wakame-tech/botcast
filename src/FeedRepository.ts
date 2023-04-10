import { SupabaseClient } from "./deps.ts";
import { Feed } from "./model.ts";
import { supabase } from "./supabase.ts";

export class FeedRepository {
  constructor(private supabase: SupabaseClient) {}

  async getAll(): Promise<Feed[]> {
    const { data, error } = await this.supabase.from("feeds");
    if (error) {
      throw error;
    }
    return data;
  }

  async create(feed: Feed): Promise<void> {
    const res = await this.supabase.from("feeds").insert(feed);
    if (res.error) {
      throw res.error;
    }
  }
}

export const feedRepository = new FeedRepository(supabase);
