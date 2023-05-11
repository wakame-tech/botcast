import { FeedRepository } from "./FeedRepository.ts";
import { service } from "./FeedService.ts";
import { Application, Item, Podcast, Router } from "./deps.ts";
import { Feed, Script } from "./model.ts";
import { supabase } from "./supabase.ts";
import { z } from "./deps.ts";

const app = new Application();
const router = new Router();

export const CreatePodcast = z.object({
  title: z.string(),
  text: z.string(),
});
export type CreatePodcast = z.infer<typeof CreatePodcast>;

router.post("/api/v1/podcasts", async (ctx) => {
  const data = await ctx.request.body({
    type: "json",
  }).value;
  const script = Script.parse(data);
  const feed = await service.postPodcast(script);
  ctx.response.status = 200;
  ctx.response.headers.set("Content-Type", "application/json");
  ctx.response.body = JSON.stringify(feed);
});

export const GenerateScript = z.object({
  title: z.string(),
  type: z.enum(["raw", "hamern"]),
  text: z.string(),
});
export type GenerateScript = z.infer<typeof GenerateScript>;

router.post("/api/v1/scripts", async (ctx) => {
  const data = await ctx.request.body({
    type: "json",
  }).value;
  const req = GenerateScript.parse(data);
  const script = service.generateScript(req);
  ctx.response.status = 200;
  ctx.response.headers.set("Content-Type", "application/json");
  ctx.response.body = JSON.stringify(script);
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
