import { z } from "../deps.ts";

export const CreatePodcast = z.object({
  title: z.string(),
  text: z.string(),
});
export type CreatePodcast = z.infer<typeof CreatePodcast>;
