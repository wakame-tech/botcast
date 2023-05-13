import { Router, Request, Response } from "./deps.ts";
import { z } from "./deps.ts";
import { feedService } from "./FeedService.ts";
import { scriptService } from "./ScriptService.ts";

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

export const Source = z.object({
  title: z.string(),
  url: z.string().nullable(),
  text: z.string(),
});
export type Source = z.infer<typeof Source>;

export const GenerateScript = z.object({
  title: z.string(),
  url: z.string().nullable(),
  sources: z.array(Source),
});
export type GenerateScript = z.infer<typeof GenerateScript>;

router.post("/api/v1/scripts", async (ctx) => {
  const req = await validate(ctx.request, GenerateScript.parse);
  const script = scriptService.generate(req.title, req.url, req.sources);
  const audio = await scriptService.synthesis(script);
  const feed = await feedService.post(script, audio);
  ctx.response = ok(ctx.request, JSON.stringify(feed));
});

router.get("/feed", async (ctx) => {
  const xml = await feedService.getFeeds();
  ctx.response = ok(ctx.request, xml, { "Content-Type": "application/xml" });
});
