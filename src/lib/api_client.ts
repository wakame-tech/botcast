import { supabase } from "@/supabase";
import createFetchClient, { type Middleware } from "openapi-fetch";
import createClient from "openapi-react-query";
import { z } from "zod";
import type { components, paths } from "./api";

const authMiddleware: Middleware = {
    async onRequest({ request }) {
        const session = await supabase.auth.getSession();
        if (session.data.session) {
            const accessToken = session.data.session.access_token;
            request.headers.set("Authorization", `Bearer ${accessToken}`);
        }
        return request;
    },
};

const fetchClient = createFetchClient<paths>({
    baseUrl: import.meta.env.VITE_API_URL,
});
fetchClient.use(authMiddleware);
export const $api = createClient(fetchClient);

export type Podcast = components["schemas"]["Podcast"];
export type Episode = components["schemas"]["Episode"];
export type Corner = components["schemas"]["Corner"];
export type User = components["schemas"]["User"];

export const taskArgsSchema = z.discriminatedUnion("type", [
    z.object({
        type: z.literal("generateAudio"),
        episodeId: z.string(),
    }),
    z.object({
        type: z.literal("evaluateTemplate"),
        template: z.record(z.any()),
        parameters: z.record(z.any()),
    }),
]);

export type TaskArgs = z.infer<typeof taskArgsSchema>;

export const taskSchema = z.object({
    result: z.record(z.any()),
    id: z.string(),
    // TODO: enum
    status: z.string(),
    args: taskArgsSchema,
    user_id: z.string().nullable(),
    execute_after: z.date().nullable(),
    executed_at: z.date().nullable(),
    executed_finished_at: z.date().nullable(),
    cron: z.string().nullable(),
});

export type Task = z.infer<typeof taskSchema>;

export const PodcastInputSchema = z.object({
    icon: z.string().regex(/\p{Emoji_Presentation}/gu),
    title: z.string(),
    description: z.string().nullable(),
});

export type PodcastInput = z.infer<typeof PodcastInputSchema>;

export const ScriptInputSchema = z.object({
    title: z.string(),
    description: z.string().nullable(),
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
