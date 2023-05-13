import { Feed, Script } from "./model.ts";
import { IPodcastRepository } from "./repo/podcast/index.ts";
import { IFeedRepository } from "./repo/feed/index.ts";
import type { Item } from "https://esm.sh/podcast@2.0.1";
import { Podcast } from "https://esm.sh/podcast@2.0.1";
import { LocalPodcastRepository } from "./repo/podcast/LocalPodcastStorage.ts";
import { MockFeedRepository } from "./repo/feed/MockFeedRepository.ts";
import { SupabasePodcastRepository } from "./repo/podcast/SupabasePodcastStorage.ts";
import { supabase } from "./lib/supabase.ts";
import { SupabaseFeedRepository } from "./repo/feed/SupabaseFeedRepository.ts";

const fromFeed = (feed: Feed): Partial<Item> => {
  return {
    guid: feed.id,
    title: feed.title,
    description: feed.description,
    url: feed.url,
    date: feed.date,
  };
};

export class FeedService {
  constructor(
    private podcastRepository: IPodcastRepository,
    private feedRepository: IFeedRepository
  ) {}

  async post(script: Script, audio: ArrayBuffer): Promise<Feed> {
    const publicUrl = await this.podcastRepository.upload(script.id, audio);
    const feed = {
      id: script.id,
      title: script.title,
      description: `URL: ${script.url}`,
      date: new Date(),
      url: publicUrl,
    };
    await this.feedRepository.create(feed).catch((e) => console.log(e));
    return feed;
  }

  async getFeeds(): Promise<string> {
    const feeds = await this.feedRepository.getAll();
    const podcast = new Podcast({
      title: "朗読fm",
      feedUrl: "",
      siteUrl: "",
      author: "w4k4me",
    });
    for (const feed of feeds) {
      podcast.addItem(fromFeed(feed));
    }
    return podcast.buildXml();
  }
}

export const feedService = new FeedService(
  new LocalPodcastRepository(),
  // new SupabasePodcastRepository(supabase),
  new MockFeedRepository()
  // new SupabaseFeedRepository(supabase)
);
