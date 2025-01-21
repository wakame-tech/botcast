import { createTRPCReact } from "@trpc/react-query";
import type { AppRouter } from "../api/src/router";
export type { User } from "@prisma/client";
export {
	MailInputSchema,
	PodcastInputSchema,
	ScriptInputSchema,
} from "../api/src/model";
export type {
	Corner,
	CornerInput,
	Episode,
	Mail,
	MailInput,
	Podcast,
	PodcastInput,
	ScriptInput,
	Sections,
	Task,
	TaskArgs,
	WithSerializedDates,
} from "../api/src/model";

export const trpc = createTRPCReact<AppRouter>();
