import { Router, Request, Response } from "./deps.ts";
import { z } from "./deps.ts";
import { feedService } from "./FeedService.ts";
import { Source } from "./model.ts";
import { taskService } from "./TaskService.ts";

export const router = new Router();

const validate = async <T>(
  req: Request,
  validator: (data: unknown) => T
): Promise<T> => {
  const data = await req.body({ type: "json" }).value;
  return validator(data);
};

const ok = (
  req: Request,
  body: string,
  headers: Record<string, string> = { "Content-Type": "application/json" }
): Response => {
  const res = new Response(req);
  res.status = 200;
  for (const [k, v] of Object.entries(headers)) {
    res.headers.set(k, v);
  }
  res.body = body;
  return res;
};

export const PostTaskRequest = z.object({
  title: z.string(),
  sources: z.array(Source),
});
export type PostTaskRequest = z.infer<typeof PostTaskRequest>;

router.post("/api/v1/feeds", async (ctx) => {
  const req = await validate(ctx.request, PostTaskRequest.parse).catch((e) => {
    ctx.response.status = 400;
    ctx.response.body = e;
  });
  if (!req) {
    return;
  }
  const task = await taskService.push(req.title, req.sources);
  ctx.response = ok(ctx.request, JSON.stringify(task));
});

router.get("/api/v1/feeds", async (ctx) => {
  const feeds = await feedService.getFeeds();
  console.log(feeds);
  ctx.response = ok(ctx.request, JSON.stringify(feeds));
});

router.get("/api/v1/feeds/:id", async (ctx) => {
  const feed = await feedService.getFeed(ctx.params.id).catch(() => null);
  if (!feed) {
    ctx.response.status = 404;
  } else {
    ctx.response = ok(ctx.request, JSON.stringify(feed));
  }
});

router.delete("/api/v1/feeds/:id", async (ctx) => {
  await feedService.delete(ctx.params.id);
});

router.get("/feed", async (ctx) => {
  const xml = await feedService.getFeedXml();
  ctx.response = ok(ctx.request, xml, { "Content-Type": "application/xml" });
});
