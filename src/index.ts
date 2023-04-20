import { FeedRepository } from "./FeedRepository.ts";
import { Application, Item, Podcast, Router } from "./deps.ts";
import { Feed, Script } from "./model.ts";
import { GenerateScript, processText } from "./script/index.ts";
import { supabase } from "./supabase.ts";
import { upload } from "./voicevox.ts";

const app = new Application();
const router = new Router();

router.post("/api/v1/podcasts", async (ctx) => {
  const data = await ctx.request.body({
    type: "json",
  }).value;
  const body = Script.parse(data);
  const feed = await upload(body);
  ctx.response.status = 200;
  ctx.response.headers.set("Content-Type", "application/json");
  ctx.response.body = JSON.stringify(feed);
});

router.post("/api/v1/scripts", async (ctx) => {
  const data = await ctx.request.body({
    type: "json",
  }).value;
  const req = GenerateScript.parse(data);
  const script = processText(req);
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
