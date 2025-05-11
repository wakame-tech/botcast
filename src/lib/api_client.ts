import { supabase } from "@/supabase";
import createFetchClient, { type Middleware } from "openapi-fetch";
import createClient from "openapi-react-query";
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
