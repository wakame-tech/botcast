import { createTRPCReact } from "@trpc/react-query";
import type { AppRouter } from "../api/src/router";
export type {
	Comment,
	Episode,
	Podcast,
	Sections,
	Task,
	TaskArgs,
	WithSerializedDates,
} from "../api/src/router";

export const trpc = createTRPCReact<AppRouter>();
