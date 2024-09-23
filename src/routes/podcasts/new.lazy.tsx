import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import { trpc } from "../../trpc.ts";
import { useState } from "react";

export const Route = createLazyFileRoute("/podcasts/new")({
	component: NewPodcast,
});

export function NewPodcast() {
	const newPodcast = trpc.newPodcast.useMutation();
	const [title, setTitle] = useState("");
	const navigate = useNavigate();

	const handleSubmit = async () => {
		const { podcast } = await newPodcast.mutateAsync({ title, icon: "🎧" });
		setTitle("");
		navigate({ to: "/podcasts/$podcastId", params: { podcastId: podcast.id } });
	};

	return (
		<>
			<h1>New Podcast</h1>
			<input
				type="text"
				value={title}
				onChange={(e) => setTitle(e.target.value)}
			/>
			<br />
			<button type="button" onClick={handleSubmit}>
				Create
			</button>
		</>
	);
}
