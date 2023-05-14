import { Feed } from "../../model.ts";
import { IFeedRepository } from "./index.ts";

export class MockFeedRepository implements IFeedRepository {
  #feeds: Feed[] = [];

  async getAll(): Promise<Omit<Feed, "script">[]> {
    return this.#feeds;
  }

  async get(id: string): Promise<Feed> {
    const feed = this.#feeds.find((feed) => feed.id === id);
    if (!feed) {
      throw "not found";
    }
    return feed;
  }

  async create(feed: Feed): Promise<void> {
    this.#feeds.push(feed);
  }
}
