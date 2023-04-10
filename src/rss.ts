import { Feed } from "./model.ts";
import { FeedRepository } from "./FeedRepository.ts";
import { Item, Podcast } from "./deps.ts";

const fromFeed = (feed: Feed): Partial<Item> => {
  return {
    guid: feed.id,
    title: feed.title,
    description: feed.description,
    url: feed.url,
    date: feed.date,
  };
};

export const getFeedXML = async (
  feedRepository: FeedRepository
): Promise<string> => {
  const feeds = await feedRepository.getAll();
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
};
