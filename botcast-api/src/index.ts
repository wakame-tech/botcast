import { PrismaClient } from "@prisma/client";
import { initTRPC } from "@trpc/server";
import { createExpressMiddleware } from "@trpc/server/adapters/express";
import express from "express";
import cors from "cors";
import { getLogger } from "log4js";
// @ts-ignore
import audit from "express-requests-logger";
import { botcastRouter } from "./botcastsRouter";
import { scriptRouter } from "./scriptRouter";

export const prisma = new PrismaClient();

export const t = initTRPC.create();

const router = t.router;

const appRouter = router({
  scripts: scriptRouter,
  botcasts: botcastRouter,
});

export type AppRouter = typeof appRouter;

const app = express();

app
  .use(
    cors({
      origin: "*",
      optionsSuccessStatus: 200,
    })
  )
  .use(
    audit({
      logger: getLogger("http"),
    })
  )
  .use(
    "/trpc",
    createExpressMiddleware({
      router: appRouter,
    })
  );

app.listen(3000);
