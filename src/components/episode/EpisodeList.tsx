import type { Episode, WithSerializedDates } from "@/trpc";
import { Link } from "@tanstack/react-router";
import dayjs from "dayjs";

type EpisodeDigest = Pick<
	WithSerializedDates<Episode>,
	"id" | "title" | "description" | "duration_sec" | "created_at"
>;

interface EpisodeListProps {
	episodes: EpisodeDigest[];
}

function EpisodeList(props: EpisodeListProps) {
	return (
		<>
			{props.episodes.map((episode) => (
				<div key={episode.id} className="p-4">
					<EpisodeListItem episode={episode} />
				</div>
			))}
		</>
	);
}

export const formatMmss = (duration_sec: number | null): string => {
	return duration_sec
		? dayjs.duration(duration_sec, "seconds").format("m:ss")
		: "--:--";
};

interface EpisodeListItemProps {
	episode: EpisodeDigest;
}

function EpisodeListItem(props: EpisodeListItemProps) {
	return (
		<>
			<div className="flex gap-2">
				<div className="grow">
					<Link
						to="/episodes/$episodeId"
						params={{ episodeId: props.episode.id }}
						className="text-xl no-underline font-bold"
					>
						{props.episode.title}
					</Link>
				</div>
				<div className="grid grid-cols-2 justify-items-end gap-2">
					<span className="text-sm text-gray">
						{dayjs(props.episode.created_at).format("YYYY-MM-DD HH:mm")}
					</span>
					<span className="font-bold">
						{formatMmss(props.episode.duration_sec)}
					</span>
				</div>
			</div>
		</>
	);
}

const components = {
	List: EpisodeList,
	Item: EpisodeListItem,
};

export default components;
