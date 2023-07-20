import { prisma, t } from "../index";
import z from "zod";

export const tasksRouter = t.router({
  retrieveTasks: t.procedure.query(async () => {
    return prisma.task.findMany();
  }),
  queueTask: t.procedure
    .input(
      z.object({
        url: z.string(),
      })
    )
    .mutation(({ input }) => {
      console.log(input.url);
    }),
});
