import { Header } from "@/components/Header.tsx";
import { EpisodePlayer } from "@/components/episode/EpisodePlayer";
import { useSession } from "@/hooks/useSession.ts";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Outlet } from "@tanstack/react-router";
import { useState } from "react";

export function App() {
	const { session } = useSession();
	const [queryClient] = useState(() => new QueryClient());

	return (
		<>
			<QueryClientProvider client={queryClient}>
				<Header session={session} />
				<Outlet />
				<div className="mb-16">
					<EpisodePlayer />
				</div>
			</QueryClientProvider>
		</>
	);
}
