import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import { trpc } from "./trpc.ts";
import { httpBatchLink } from "@trpc/client";

function Episodes() {
  const episodeQuery = trpc.episodes.useQuery();

  return (
    <div>
      {JSON.stringify(episodeQuery.data?.episodes)}
    </div>
  );
}

function App() {
  const [queryClient] = useState(() => new QueryClient());
  const [trpcClient] = useState(() =>
    trpc.createClient({
      links: [
        httpBatchLink({
          url: "/trpc",
          async headers() {
            return {};
          },
        }),
      ],
    })
  );

  return (
    <>
      <trpc.Provider client={trpcClient} queryClient={queryClient}>
        <QueryClientProvider client={queryClient}>
          <Episodes />
        </QueryClientProvider>
      </trpc.Provider>
    </>
  );
}

export default App;
