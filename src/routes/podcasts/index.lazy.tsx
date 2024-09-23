import { createLazyFileRoute, Link } from "@tanstack/react-router";
import { trpc } from "../../trpc.ts";

export const Route = createLazyFileRoute("/podcasts/")({
	component: Podcasts,
});

function Podcasts() {
	const getPodcasts = trpc.podcasts.useQuery();
	const podcasts = getPodcasts.data?.podcasts ?? [];

	return (
		<>
			<Link to="/podcasts/new">New Podcast</Link>
			<h1>Podcasts</h1>
			{podcasts.map((podcast) => (
				<div key={podcast.id}>
					<Link to="/podcasts/$podcastId" params={{ podcastId: podcast.id }}>
						<h3>{podcast.title}</h3>
					</Link>
				</div>
			))}
		</>
	);
}
