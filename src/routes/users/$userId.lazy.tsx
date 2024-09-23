import Podcast from "@/components/podcast/PodcastList";
import { trpc } from "@/trpc";
import { Link, createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/users/$userId")({
	component: User,
});

function User() {
	const getMe = trpc.me.useQuery();
	const getPodcasts = trpc.podcasts.useQuery();
	const podcasts = getPodcasts.data?.podcasts ?? [];

	if (!getMe.data) {
		return <div>loading...</div>;
	}

	const user = getMe.data.user;

	return (
		<>
			<h1>{user.name}</h1>
			<h2>Podcasts</h2>
			<Link to="/podcasts/new">New Podcast</Link>
			<Podcast.List podcasts={podcasts} />

			<h2>Tasks</h2>
			<Link to="/tasks">Tasks</Link>
		</>
	);
}
