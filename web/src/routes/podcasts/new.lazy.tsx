import { PodcastForm } from "@/components/podcast/PodcastForm";
import { useSession } from "@/hooks/useSession";
import type { PodcastInput } from "@/lib/api_client";
import { $cms, toRecordData } from "@/lib/cms_client";
import { useNavigate } from "@tanstack/react-router";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/new")({
	component: NewPodcast,
});

export function NewPodcast() {
	const navigate = useNavigate();
	const { userId } = useSession();
	const newPodcast = $cms.useMutation("post", "/records/{collectionId}");

	const handleSubmit = async (values: PodcastInput) => {
		await newPodcast.mutateAsync({
			params: { path: { collectionId: "podcasts" } },
			body: {
				data: toRecordData({
					title: values.title,
					description: values.description,
					icon: values.icon,
					user_id: userId ?? "",
				}),
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
