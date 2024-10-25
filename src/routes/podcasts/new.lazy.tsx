import { PodcastForm } from "@/components/podcast/PodcastForm";
import type { PodcastEditFormValues } from "@/components/podcast/PodcastForm";
import { MANUSCRIPT_TEMPLATE } from "@/components/script/ScriptForm";
import { Textarea } from "@/components/ui/textarea";
import { trpc } from "@/trpc.ts";
import { useNavigate } from "@tanstack/react-router";
import { createLazyFileRoute } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createLazyFileRoute("/podcasts/new")({
	component: NewPodcast,
});

export function NewPodcast() {
	const navigate = useNavigate();
	const newPodcast = trpc.newPodcast.useMutation();
	const [template, setTemplate] = useState(MANUSCRIPT_TEMPLATE);

	const handleSubmit = async (values: PodcastEditFormValues) => {
		const { podcast } = await newPodcast.mutateAsync({
			title: values.title,
			template,
			icon: "🎧",
			weekDay: values.weekDay,
			hour: values.hour,
		});
		navigate({ to: "/podcasts/$podcastId", params: { podcastId: podcast.id } });
	};

	return (
		<>
			<h1>新しいポッドキャスト</h1>
			<PodcastForm onSubmit={handleSubmit}>
				<Textarea
					rows={10}
					placeholder="template"
					value={template}
					onChange={(e) => setTemplate(e.target.value)}
				/>
			</PodcastForm>
		</>
	);
}
