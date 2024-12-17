import { Header } from "@/components/Header.tsx";
import { EpisodePlayer } from "@/components/episode/EpisodePlayer";
import { useSession } from "@/hooks/useSession.ts";
import { supabase } from "@/supabase.ts";
import { trpc } from "@/trpc.ts";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Outlet, useLocation, useNavigate } from "@tanstack/react-router";
import { httpBatchLink } from "@trpc/client";
import { useEffect, useState } from "react";

let token = "";

export function App() {
	const location = useLocation();
	const navigate = useNavigate();
	const { session, setSession } = useSession();
	const [queryClient] = useState(() => new QueryClient());
	const [trpcClient] = useState(() =>
		trpc.createClient({
			links: [
				httpBatchLink({
					url: `${import.meta.env.DEV ? import.meta.env.VITE_API_URL : ""}/trpc`,
					async headers() {
						return {
							Authorization: token,
						};
					},
				}),
			],
		}),
	);

	useEffect(() => {
		const {
			data: { subscription },
		} = supabase.auth.onAuthStateChange((event, session) => {
			setSession(session);
			if (session) {
				token = session.access_token;
				if (event === "SIGNED_IN" && location.pathname === "/signin") {
					navigate({ to: "/" });
				}
			}
		});

		return () => subscription.unsubscribe();
	}, [setSession, navigate, location]);

	return (
		<>
			<trpc.Provider client={trpcClient} queryClient={queryClient}>
				<QueryClientProvider client={queryClient}>
					<Header session={session} />
					<Outlet />
					<div className="mb-16">
						<EpisodePlayer />
					</div>
				</QueryClientProvider>
			</trpc.Provider>
		</>
	);
}
