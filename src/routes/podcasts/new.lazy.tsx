import { PodcastForm } from "@/components/podcast/PodcastForm";
import type { Podcast } from "@prisma/client";
import { useNavigate } from "@tanstack/react-router";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/new")({
	component: NewPodcast,
});

export function NewPodcast() {
	const navigate = useNavigate();

	const onCreate = (podcast: Podcast) => {
		navigate({ to: "/podcasts/$podcastId", params: { podcastId: podcast.id } });
	};

	return (
		<>
			<h1>新しいポッドキャスト</h1>
			<PodcastForm onCreate={onCreate} />
		</>
	);
}
