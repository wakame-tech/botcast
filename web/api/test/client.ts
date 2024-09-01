import { AppRouter } from "../src/router.ts";
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

// const tasks = await client.tasks.query();
// console.log(JSON.stringify(tasks));

const res = await client.newEpisode.mutate({
    title: "test2",
    url: "https://zenn.dev/koko_u/scraps/1d8c7d1b5e3c6f",
});
console.log(JSON.stringify(res));
