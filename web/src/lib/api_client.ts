import { getToken } from "@/hooks/useSession";
import createFetchClient, { type Middleware } from "openapi-fetch";
import createClient from "openapi-react-query";
import { z } from "zod";
import type { components, paths } from "./api";

const authMiddleware: Middleware = {
	async onRequest({ request }) {
		const token = getToken();
		if (token) {
			request.headers.set("Authorization", `Bearer ${token}`);
		}
		return request;
	},
};

const fetchClient = createFetchClient<paths>({
	baseUrl: import.meta.env.VITE_API_URL,
});
fetchClient.use(authMiddleware);
export const $api = createClient(fetchClient);

export type Section = components["schemas"]["Section"];
export type User = components["schemas"]["User"];

export const PodcastInputSchema = z.object({
	icon: z.string().regex(/\p{Emoji_Presentation}/gu),
	title: z.string(),
	description: z.string(),
});

export type PodcastInput = z.infer<typeof PodcastInputSchema>;

export const ScriptInputSchema = z.object({
	title: z.string(),
	description: z.string(),
	template: z.string(),
});

export type ScriptInput = z.infer<typeof ScriptInputSchema>;

export const CornerInputSchema = z.object({
	title: z.string(),
	description: z.string().nullable(),
	requesting_mail: z.boolean(),
	mail_schema: z.string(),
});

export type CornerInput = z.infer<typeof CornerInputSchema>;

export const MailInputSchema = z.object({
	body: z.record(z.unknown()),
});

export type MailInput = z.infer<typeof MailInputSchema>;
