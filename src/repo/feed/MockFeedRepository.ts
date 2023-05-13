import { Feed } from "../../model.ts";
import { IFeedRepository } from "./index.ts";

export class MockFeedRepository implements IFeedRepository {
  #feeds: Feed[] = [];

  async getAll(): Promise<Feed[]> {
    return this.#feeds;
  }

  async create(feed: Feed): Promise<void> {
    this.#feeds.push(feed);
  }
}
