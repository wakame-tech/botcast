import { Feed, FeedDigest } from "../../model.ts";
import { IFeedRepository } from "./index.ts";

export class MockFeedRepository implements IFeedRepository {
  #feeds: Record<string, Feed> = {};

  async getAll(): Promise<FeedDigest[]> {
    return Object.values(this.#feeds);
  }

  async get(id: string): Promise<Feed> {
    const feed = this.#feeds[id];
    if (!feed) {
      throw "not found";
    }
    return feed;
  }

  async create(feed: Feed): Promise<void> {
    this.#feeds[feed.id] = feed;
  }

  async delete(id: string): Promise<void> {
    delete this.#feeds[id];
  }
}
