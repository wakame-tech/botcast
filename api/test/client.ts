import { AppRouter } from "../src/index.ts";
import { createTRPCProxyClient, httpLink } from "@trpc/client";

const url = "http://localhost:1234/trpc";
// const url = "http://botcast.deno.dev/trpc";

const client = createTRPCProxyClient<AppRouter>({
    links: [
        httpLink({
            url,
        }),
    ],
});

const users = await client.users.query();
console.log(JSON.stringify(users));
