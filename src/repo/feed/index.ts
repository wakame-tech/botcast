import { Feed } from "../../model.ts";

export interface IFeedRepository {
  getAll(): Promise<Feed[]>;
  create(feed: Feed): Promise<void>;
}
