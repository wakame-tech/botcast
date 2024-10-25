import type { MappedString } from "@/lib/utils";
import type { Podcast } from "@prisma/client";
import { Link } from "@tanstack/react-router";
import { Button } from "../ui/button";

interface PodcastListProps {
	podcasts: MappedString<Podcast, "created_at">[];
}

function PodcastList(props: PodcastListProps) {
	return (
		<>
			<div className="pb-2 flex items-center">
				<div className="flex-grow" />
				<Link to="/podcasts/new">
					<Button>新規作成</Button>
				</Link>
			</div>
			{props.podcasts.map((podcast) => (
				<div key={podcast.id} className="p-2 bg-teal-50">
					<PodcastListItem podcast={podcast} />
				</div>
			))}
		</>
	);
}

interface PodcastListItemProps {
	podcast: MappedString<Podcast, "created_at">;
}

function PodcastListItem(props: PodcastListItemProps) {
	return (
		<>
			<Link
				to="/podcasts/$podcastId"
				params={{ podcastId: props.podcast.id }}
				className="no-underline"
			>
				<span className="text-xl font-bold">{props.podcast.title}</span>
			</Link>
		</>
	);
}

const components = {
	List: PodcastList,
	Item: PodcastListItem,
};

export default components;
