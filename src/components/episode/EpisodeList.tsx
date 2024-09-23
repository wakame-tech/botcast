import type { Episode } from "@prisma/client";
import { Link } from "@tanstack/react-router";
import dayjs from "dayjs";

interface EpisodeListProps {
	podcastId: string;
	episodes: Episode[];
	onClickDelete: () => Promise<void>;
}

function EpisodeList(props: EpisodeListProps) {
	return (
		<>
			<div className="pb-2 flex justify-end">
				<Link
					to="/podcasts/$podcastId/new"
					params={{ podcastId: props.podcastId }}
					className="p-2 no-underline rounded-md bg-teal-500 text-white"
				>
					新規作成
				</Link>
			</div>

			{props.episodes.map((episode) => (
				<div key={episode.id} className="p-2 bg-teal-100">
					<EpisodeListItem episode={episode} />
				</div>
			))}
			<button type="button" onClick={props.onClickDelete}>
				Delete
			</button>
		</>
	);
}

interface EpisodeListItemProps {
	episode: Episode;
}

function EpisodeListItem(props: EpisodeListItemProps) {
	return (
		<>
			<Link
				to="/episodes/$episodeId"
				params={{ episodeId: props.episode.id }}
				className="no-underline"
			>
				<span className="text-xl font-bold">{props.episode.title}</span>
			</Link>
			<p className="p-0 m-0">
				<span className="text-xs text-gray">
					{dayjs(props.episode.created_at).format("YYYY-MM-DD HH:mm")}
				</span>
			</p>
		</>
	);
}

const components = {
	List: EpisodeList,
	Item: EpisodeListItem,
};

export default components;
