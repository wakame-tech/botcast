import { SupabaseClient } from "../../deps.ts";
import { Feed } from "../../model.ts";
import { IFeedRepository } from "./index.ts";

export class SupabaseFeedRepository implements IFeedRepository {
  constructor(private supabase: SupabaseClient) {}

  async getAll(): Promise<Omit<Feed, "script">[]> {
    const { data, error } = await this.supabase
      .from("feeds")
      .select("id,date,title,description,url");
    if (error) {
      throw error;
    }
    return data;
  }

  async get(id: string): Promise<Feed> {
    const { data, error } = await this.supabase
      .from("feeds")
      .select("*")
      .filter("id", "eq", id)
      .limit(1);
    if (error) {
      throw error;
    }
    if (!data[0]) {
      throw "not found";
    }
    return data[0];
  }

  async create(feed: Feed): Promise<void> {
    const res = await this.supabase.from("feeds").upsert(feed);
    if (res.error) {
      throw res.error;
    }
  }
}
