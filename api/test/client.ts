import { AppRouter } from "../src/index.ts";
import { createTRPCProxyClient, httpLink } from "@trpc/client";

const client = createTRPCProxyClient<AppRouter>({
    links: [
        httpLink({
            url: "http://localhost:1234/trpc",
        }),
    ],
});

const res = await client.ping.query();
console.log(JSON.stringify(res));
