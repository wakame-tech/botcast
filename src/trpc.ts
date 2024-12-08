import { createTRPCReact } from "@trpc/react-query";
import type { AppRouter } from "../api/src/router";
export { PodcastInputSchema } from "../api/src/model";
export type {
	Comment,
	Episode,
	Podcast,
	PodcastInput,
	Sections,
	Task,
	TaskArgs,
	WithSerializedDates,
} from "../api/src/model";

export const trpc = createTRPCReact<AppRouter>();
