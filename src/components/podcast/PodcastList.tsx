import type { Podcast } from "@prisma/client";
import { Link } from "@tanstack/react-router";

interface PodcastListProps {
	podcasts: Podcast[];
}

function PodcastList(props: PodcastListProps) {
	return (
		<>
			{props.podcasts.map((podcast) => (
				<div key={podcast.id}>
					<Link to="/podcasts/$podcastId" params={{ podcastId: podcast.id }}>
						<h3>{podcast.title}</h3>
					</Link>
				</div>
			))}
		</>
	);
}

const components = {
	List: PodcastList,
};

export default components;
