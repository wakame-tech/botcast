import { AppRouter } from "../src/router.ts";
import { createTRPCProxyClient, httpLink } from "@trpc/client";

const url = "http://localhost:1234/trpc";
// const url = "http://botcast.deno.dev/trpc";

let token = "";

const client = createTRPCProxyClient<AppRouter>({
  links: [
    httpLink({
      url,
      headers: () => {
        return token ? { Authorization: token } : {};
      },
    }),
  ],
});

const res = await client.signIn.query({
  email: "kamata1919@gmail.com",
  password: "aaaa1234",
});
token = res;

const podcastId = "82b1f069-ed6e-4fdd-9546-4ac120f0ecfd";

const res2 = await client.newCorner.mutate({
  podcastId,
  title: "好きな食べ物を教えて下さい",
  description: "このコーナーでは、リスナーから好きな食べ物を募集します。好きな食べ物と好きになったきっかけやエピソードがあれば一緒に教えてください。",
  mail_schema: JSON.stringify({
    type: "object",
    properties: {
      name: { type: "string" },
      content: { type: "string" },
    },
    required: ["name", "content"],
  }),
});
console.log(res2);
