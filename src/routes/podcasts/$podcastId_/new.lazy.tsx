import { trpc } from "@/trpc.ts";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createLazyFileRoute("/podcasts/$podcastId/new")({
	component: NewEpisode,
});

export function NewEpisode() {
	const { podcastId } = Route.useParams();
	const newEpisode = trpc.newEpisode.useMutation();
	const [title, setTitle] = useState("");
	const [url, setUrl] = useState("");
	const navigate = useNavigate();

	const handleSubmit = async () => {
		const { episode } = await newEpisode.mutateAsync({ podcastId, title, url });
		setTitle("");
		setUrl("");
		navigate({ to: "/episodes/$episodeId", params: { episodeId: episode.id } });
	};

	return (
		<>
			<h1>New Episode</h1>
			<input
				type="text"
				value={title}
				onChange={(e) => setTitle(e.target.value)}
				placeholder="Title"
			/>
			<br />
			<input
				type="text"
				value={url}
				onChange={(e) => setUrl(e.target.value)}
				placeholder="URL"
			/>
			<br />
			<button type="button" onClick={handleSubmit}>
				Create
			</button>
		</>
	);
}
