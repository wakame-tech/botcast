import { t } from "./index";
import z from "zod";
import { Source } from "./model";

export const scriptRouter = t.router({
  registerSynthesisScript: t.procedure
    .meta({ openapi: { method: "POST", path: "/scripts" } })
    .input(
      z.object({
        title: z.string(),
        sources: z.array(Source),
      })
    )
    .query(({ input }) => {
      return {};
    }),
});
