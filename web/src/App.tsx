import { Outlet } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/router-devtools'
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import { trpc } from "./trpc.ts";
import { httpBatchLink } from "@trpc/client";
import { Header } from './components/Header.tsx';

export function App() {
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
                    {import.meta.env.DEV && <TanStackRouterDevtools />}
                </QueryClientProvider>
            </trpc.Provider>
        </>
    );
}
