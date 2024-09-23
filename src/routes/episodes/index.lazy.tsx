import { createLazyFileRoute, Link } from "@tanstack/react-router";
import { trpc } from "../../trpc.ts";

export const Route = createLazyFileRoute("/episodes/")({
	component: Episodes,
});

function Episodes() {
	const episodesQuery = trpc.episodes.useQuery();
	const episodes = episodesQuery.data?.episodes ?? [];

	return (
		<div>
			{episodes.map((episode) => (
				<div key={episode.id}>
					<Link to="/episodes/$episodeId" params={{ episodeId: episode.id }}>
						<h3>{episode.title}</h3>
					</Link>
				</div>
			))}
		</div>
	);
}
