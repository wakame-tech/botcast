import { Transcript } from "@/components/episode/Transcript";
import { GenerateScriptForm } from "@/components/job/GenerateScriptForm";
import { JobButton } from "@/components/job/JobButton";
import { EpisodeMails } from "@/components/mail/EpisodeMails";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { usePlayer } from "@/hooks/usePlayer";
import { useSession } from "@/hooks/useSession";
import {
	$cms,
	CMS_TENANT,
	fetchCmsFile,
	isOwner,
	recordToEpisode,
} from "@/lib/cms_client";
import { useQuery } from "@tanstack/react-query";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import Parser from "srt-parser-2";
import type { Line } from "srt-parser-2";

export const Route = createLazyFileRoute(
	"/podcasts/$podcastId/episodes/$episodeId",
)({
	component: Episode,
});

const fetchSrt = async (url: string): Promise<Line[]> => {
	const srt = await (await fetchCmsFile(url)).text();
	const parser = new Parser();
	return parser.fromSrt(srt);
};

function Episode() {
	const navigate = useNavigate();
	const { podcastId, episodeId } = Route.useParams();
	const { userId } = useSession();
	const { data: episodeRecord } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
		{
			params: {
				path: {
					tenantId: CMS_TENANT,
					collectionId: "episodes",
					recordId: episodeId,
				},
			},
		},
	);
	const episode = episodeRecord ? recordToEpisode(episodeRecord) : null;

	const { data } = useQuery({
		queryKey: ["lines", episode?.srt_url],
		queryFn: () => {
			const url = episode?.srt_url;
			if (url) {
				return fetchSrt(url);
			}
		},
		enabled: !!episode?.srt_url,
	});
	const lines = data ?? [];

	const deleteEpisode = $cms.useMutation(
		"delete",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
	);

	const { state, setEpisode, playOrPause, playFrom } = usePlayer();

	if (!episode) {
		return <div>error</div>;
	}

	const isPlayingEpisode = state.isPlaying && episode.id === state.episode?.id;
	const editable = isOwner(episode, userId);
	const generated = !!episode.audio_url && !!episode.srt_url;

	const handleDelete = async () => {
		await deleteEpisode.mutateAsync({
			params: {
				path: {
					tenantId: CMS_TENANT,
					collectionId: "episodes",
					recordId: episodeId,
				},
			},
		});
		navigate({
			to: "/podcasts/$podcastId",
			params: { podcastId },
		});
	};

	const icon = isPlayingEpisode ? "i-solar:pause-bold" : "i-solar:play-bold";
	const iconColor = episode ? "text-white" : "text-gray-300";

	const onClickPlayButton = () => {
		setEpisode(episode);
		playOrPause();
	};

	return (
		<>
			<Card>
				<CardHeader>
					<CardTitle>
						<div className="flex content-center gap-2">
							<div
								className="bg-teal-400 w-16 h-16 rounded-xl flex items-center justify-center"
								onClick={() => onClickPlayButton()}
								onKeyDown={() => onClickPlayButton()}
							>
								<p className={`${icon} ${iconColor}`} />
							</div>
							<span className="grow pl-2 text-xl font-bold">
								{episode.title}
							</span>
							{editable && (
								<div className="text-sm">
									<Button className="bg-red-500" onClick={handleDelete}>
										削除
									</Button>
								</div>
							)}
						</div>
					</CardTitle>
				</CardHeader>
				<CardContent>
					{generated ? (
						<>
							<Transcript
								lines={lines}
								sections={episode.sections}
								isCurrentEpisode={episode.id === state.episode?.id}
								seconds={state.seconds}
								onClickLine={(seconds) => playFrom(episode, seconds)}
							/>
							{editable && (
								<details className="pt-4 text-sm">
									<summary className="cursor-pointer text-gray-500">
										作り直す
									</summary>
									<GenerateForms episodeId={episodeId} />
								</details>
							)}
						</>
					) : (
						editable && <GenerateForms episodeId={episodeId} />
					)}
				</CardContent>
			</Card>
			<EpisodeMails episodeId={episodeId} />
		</>
	);
}

/** 音声の生成と台本の生成 */
function GenerateForms({ episodeId }: { episodeId: string }) {
	return (
		<div className="py-2 flex flex-col gap-2">
			<JobButton
				label="音声を生成"
				args={{ type: "generateAudio", episodeId }}
			/>
			<GenerateScriptForm episodeId={episodeId} />
		</div>
	);
}
