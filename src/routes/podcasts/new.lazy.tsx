import { createLazyFileRoute } from "@tanstack/react-router";
import { trpc } from "../../trpc.ts";
import { useState } from "react";

export const Route = createLazyFileRoute("/podcasts/new")({
	component: NewPodcast,
});

export function NewPodcast() {
	const newPodcast = trpc.newPodcast.useMutation();
	const [title, setTitle] = useState("");

	const handleSubmit = async () => {
		await newPodcast.mutateAsync({ title });
		setTitle("");
	};

	return (
		<div>
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
		</div>
	);
}
