import { PrismaClient } from "@prisma/client";
import { initTRPC } from "@trpc/server";
import { createExpressMiddleware } from "@trpc/server/adapters/express";
import express from "express";
import cors from "cors";
import { getLogger } from "log4js";
// @ts-ignore
import audit from "express-requests-logger";

export const prisma = new PrismaClient();

const t = initTRPC.create();

const router = t.router;
const publicProcedure = t.procedure;

const appRouter = router({
  users: publicProcedure.query(async () => {
    const users = await prisma.user.findMany();
    return users;
  }),
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
