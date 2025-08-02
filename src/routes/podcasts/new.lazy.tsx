import { PodcastForm } from "@/components/podcast/PodcastForm";
import { $api, type PodcastInput } from "@/lib/api_client";
import { useNavigate } from "@tanstack/react-router";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/new")({
	component: NewPodcast,
});

export function NewPodcast() {
	const navigate = useNavigate();
	const newPodcast = $api.useMutation("post", "/podcasts");

	const handleSubmit = async (values: PodcastInput) => {
		await newPodcast.mutateAsync({
			body: {
				title: values.title,
				description: values.description,
				icon: "🎧",
			},
		});
		navigate({ to: "/podcasts" });
	};

	return (
		<>
			<h1>新しいポッドキャスト</h1>
			<PodcastForm onSubmit={handleSubmit} />
		</>
	);
}
