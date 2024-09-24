import { EpisodeForm } from "@/components/episode/EpisodeForm";
import type { Episode } from "@prisma/client";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/$podcastId/new")({
	component: NewEpisode,
});

export function NewEpisode() {
	const { podcastId } = Route.useParams();
	const navigate = useNavigate();

	const onCreate = (episode: Episode) => {
		navigate({ to: "/episodes/$episodeId", params: { episodeId: episode.id } });
	};

	return (
		<>
			<h1>New Episode</h1>
			<EpisodeForm podcastId={podcastId} onCreate={onCreate} />
		</>
	);
}
