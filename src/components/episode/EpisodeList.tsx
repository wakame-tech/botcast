import type { Episode, WithSerializedDates } from "@/trpc";
import { Link } from "@tanstack/react-router";
import dayjs from "dayjs";
import { Button } from "../ui/button";

type EpisodeDigest = Pick<
	WithSerializedDates<Episode>,
	"id" | "title" | "description" | "duration_sec" | "created_at"
>;

interface EpisodeListProps {
	podcastId: string;
	episodes: EpisodeDigest[];
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
	episode: EpisodeDigest;
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
				{props.episode.duration_sec && (
					<span className="text-xs text-gray">
						{dayjs
							.duration(props.episode.duration_sec, "seconds")
							.format("m:ss")}
					</span>
				)}
			</p>
		</>
	);
}

const components = {
	List: EpisodeList,
	Item: EpisodeListItem,
};

export default components;
