import { Feed } from "../../model.ts";

export interface IFeedRepository {
  getAll(): Promise<Omit<Feed, "script">[]>;
  get(id: string): Promise<Feed>;
  create(feed: Feed): Promise<void>;
}
