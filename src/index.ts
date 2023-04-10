import { FeedRepository } from "./FeedRepository.ts";
import { Application, Item, Podcast, Router } from "./deps.ts";
import { Feed } from "./model.ts";
import { supabase } from "./supabase.ts";
import { upload } from "./voicevox.ts";

const app = new Application();
const router = new Router();

interface CreatePodcastDto {
  title: string;
  text: string;
}

router.post("/api/v1/podcasts", async (ctx) => {
  console.log(await ctx.request.body({ type: "text" }).value);
  const body = (await ctx.request.body({
    type: "json",
  }).value) as CreatePodcastDto;
  const feed = await upload(body.title, body.text);
  ctx.response.status = 200;
  ctx.response.headers.set("Content-Type", "application/json");
  ctx.response.body = JSON.stringify(feed);
});

const fromFeed = (feed: Feed): Partial<Item> => {
  return {
    guid: feed.id,
    title: feed.title,
    description: feed.description,
    url: feed.url,
    date: feed.date,
  };
};

export const getFeedXML = (feeds: Feed[]): string => {
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

router.get("/feed", async (ctx) => {
  const feedRepository = new FeedRepository(supabase);
  const feeds = await feedRepository.getAll();
  const xml = getFeedXML(feeds);

  ctx.response.status = 200;
  ctx.response.body = xml;
  ctx.response.headers.set("Content-Type", "application/xml");
});

app.use(router.routes());
app.use(router.allowedMethods());
app.listen({ port: 8000 });
