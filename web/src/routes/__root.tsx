import { createRootRoute, Link, Outlet } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/router-devtools'
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import { trpc } from "../trpc.ts";
import { httpBatchLink } from "@trpc/client";

const Header = () => {
    return (
        <>
            <div className="p-2 flex gap-2">
                <Link to="/" className="[&.active]:font-bold">
                    Home
                </Link>{' '}
            </div>
            <hr />
        </>
    )
}

export const Route = createRootRoute({
    component: () => {
        const [queryClient] = useState(() => new QueryClient());
        const [trpcClient] = useState(() =>
            trpc.createClient({
                links: [
                    httpBatchLink({
                        url: `${import.meta.env.DEV ? import.meta.env.VITE_API_URL : ''}/trpc`,
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
                        <Header />
                        <Outlet />
                        <TanStackRouterDevtools />
                    </QueryClientProvider>
                </trpc.Provider>
            </>
        );
    },
})
