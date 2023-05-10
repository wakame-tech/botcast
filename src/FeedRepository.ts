import { SupabaseClient } from "./deps.ts";
import { Feed } from "./model.ts";

export interface IFeedRepository {
  getAll(): Promise<Feed[]>;
  create(feed: Feed): Promise<void>;
}

export class FeedRepository implements IFeedRepository {
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

export class MockFeedRepository implements IFeedRepository {
  async getAll(): Promise<Feed[]> {
    return [];
  }

  async create(feed: Feed): Promise<void> {}
}
