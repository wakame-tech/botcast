import { Outlet, useNavigate } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/router-devtools'
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { trpc } from "./trpc.ts";
import { httpBatchLink } from "@trpc/client";
import { Header } from './components/Header.tsx';
import { supabase } from './supabase.ts';
import { useSession } from './hooks/useSession.ts';

let token = '';

export function App() {
    const navigate = useNavigate();
    const { session, setSession } = useSession();
    const [queryClient] = useState(() => new QueryClient());
    const [trpcClient] = useState(() =>
        trpc.createClient({
            links: [
                httpBatchLink({
                    url: `${import.meta.env.DEV ? import.meta.env.VITE_API_URL : ''}/trpc`,
                    async headers() {
                        return {
                            Authorization: token,
                        };
                    },
                }),
            ],
        })
    );

    useEffect(() => {
        const { data: { subscription } } = supabase.auth.onAuthStateChange(
            (event, session) => {
                console.log(event, session);
                if (session) {
                    setSession(session);
                    token = session.access_token;
                }
            },
        );

        return () => subscription.unsubscribe();
    }, [setSession]);

    return (
        <>
            <trpc.Provider client={trpcClient} queryClient={queryClient}>
                <QueryClientProvider client={queryClient}>
                    <Header session={session} />
                    <Outlet />
                    {import.meta.env.DEV && <TanStackRouterDevtools />}
                </QueryClientProvider>
            </trpc.Provider>
        </>
    );
}
