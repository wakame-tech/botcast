import { createLazyFileRoute, Link } from "@tanstack/react-router";
import { trpc } from "../../trpc.ts";

export const Route = createLazyFileRoute("/podcasts/$podcastId")({
	component: Podcast,
});

export function Podcast() {
	const { podcastId } = Route.useParams();
	const getPodcast = trpc.podcast.useQuery({ id: podcastId });
	const deletePodcast = trpc.deletePodcast.useMutation();
	const podcast = getPodcast.data?.podcast;

	if (!podcast) {
		return <div>not found</div>;
	}

	const handleDelete = async () => {
		await deletePodcast.mutateAsync({ id: podcastId });
	};

	return (
		<div>
			<h1>{podcast.title}</h1>
			<Link to="/podcasts/$podcastId/new" params={{ podcastId }}>
				New Episode
			</Link>
			{podcast.episodes.map((episode) => (
				<div key={episode.id}>
					<Link to="/episodes/$episodeId" params={{ episodeId: episode.id }}>
						<h3>{episode.title}</h3>
					</Link>
				</div>
			))}
			<button type="button" onClick={handleDelete}>
				Delete
			</button>
		</div>
	);
}
