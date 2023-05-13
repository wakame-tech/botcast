import { Application } from "./deps.ts";
import { router } from "./routes.ts";

const app = new Application();

app.use(async (ctx, next) => {
  console.log(`[${ctx.request.method}] ${ctx.request.url}`);
  await next();
});

await app.use(router.routes()).listen({ port: 8000 });
