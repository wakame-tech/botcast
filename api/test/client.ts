import { AppRouter } from "../src/router.ts";
import { createTRPCProxyClient, httpLink } from "@trpc/client";

const url = "http://localhost:1234/trpc";
// const url = "http://botcast.deno.dev/trpc";

let token = "";

const _client = createTRPCProxyClient<AppRouter>({
    links: [
        httpLink({
            url,
            headers: () => {
                return token ? { Authorization: token } : {};
            },
        }),
    ],
});

// const tasks = await client.tasks.query();
// console.log(JSON.stringify(tasks));
