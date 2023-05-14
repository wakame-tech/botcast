import { taskService } from "./TaskService.ts";
import { Application } from "./deps.ts";
import { router } from "./routes.ts";

taskService.poll();

const app = new Application();
await app
  .use(async (ctx, next) => {
    console.log(`[${ctx.request.method}] ${ctx.request.url}`);
    await next();
  })
  .use((ctx, next) => {
    ctx.response.headers.set("Access-Control-Allow-Origin", "*");
    return next();
  })
  .use(router.routes())
  .listen({ port: 8000 });
