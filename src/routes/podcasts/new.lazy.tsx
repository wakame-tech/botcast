import { PodcastForm } from "@/components/podcast/PodcastForm";
import { trpc } from "@/trpc.ts";
import type { PodcastInput } from "@/trpc.ts";
import { useNavigate } from "@tanstack/react-router";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/new")({
	component: NewPodcast,
});

export function NewPodcast() {
	const navigate = useNavigate();
	const newPodcast = trpc.newPodcast.useMutation();

	const handleSubmit = async (values: PodcastInput) => {
		const { podcast } = await newPodcast.mutateAsync({
			title: values.title,
			description: values.description,
			icon: "🎧",
		});
		navigate({ to: "/podcasts/$podcastId", params: { podcastId: podcast.id } });
	};

	return (
		<>
			<h1>新しいポッドキャスト</h1>
			<PodcastForm onSubmit={handleSubmit} />
		</>
	);
}
