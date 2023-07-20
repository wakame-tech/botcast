import { t } from "./index";
import z from "zod";

export const botcastRouter = t.router({
  synthesisBotcastFromScript: t.procedure
    .input(
      z.object({
        script: z.string(),
      })
    )
    .query(({ input }) => {
      return {};
    }),
});
