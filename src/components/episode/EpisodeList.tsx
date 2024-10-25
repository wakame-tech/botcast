import type { MappedString } from "@/lib/utils";
import type { Episode } from "@prisma/client";
import { Link } from "@tanstack/react-router";
import dayjs from "dayjs";
import { Button } from "../ui/button";

interface EpisodeListProps {
	podcastId: string;
	episodes: MappedString<Episode, "created_at">[];
	onClickDelete: () => Promise<void>;
}

function EpisodeList(props: EpisodeListProps) {
	return (
		<>
			<div className="flex items-center">
				<p className="text-xl font-bold">エピソード</p>
				<div className="flex-grow" />
				<Button className="bg-red-400" onClick={props.onClickDelete}>
					削除
				</Button>
			</div>
			{props.episodes.map((episode) => (
				<div key={episode.id} className="p-2 bg-teal-50">
					<EpisodeListItem episode={episode} />
				</div>
			))}
		</>
	);
}

interface EpisodeListItemProps {
	episode: MappedString<Episode, "created_at">;
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
