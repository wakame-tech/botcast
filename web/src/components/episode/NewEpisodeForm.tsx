import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { $cms, CMS_TENANT, type Podcast, toRecordData } from "@/lib/cms_client";
import { useEnqueueJob } from "@/lib/worker_client";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";

interface NewEpisodeFormProps {
	podcasts: Podcast[];
}

/**
 * 番組を選んでエピソードを作り、台本生成（任意で音声生成まで）のジョブを投入して
 * 作ったエピソードの画面へ移動するフォーム
 */
export function NewEpisodeForm({ podcasts }: NewEpisodeFormProps) {
	const navigate = useNavigate();
	const [podcastId, setPodcastId] = useState(podcasts[0]?.id ?? "");
	const [title, setTitle] = useState("");
	const [prompt, setPrompt] = useState("");
	const [generateAudio, setGenerateAudio] = useState(true);
	const createEpisode = $cms.useMutation(
		"post",
		"/t/{tenantId}/records/{collectionId}",
	);
	const enqueue = useEnqueueJob();
	const [error, setError] = useState<string | null>(null);
	const pending = createEpisode.isPending || enqueue.isPending;

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setError(null);
		const record = await createEpisode
			.mutateAsync({
				params: { path: { tenantId: CMS_TENANT, collectionId: "episodes" } },
				body: {
					data: toRecordData({
						title,
						podcast_id: podcastId,
						sections: [],
					}),
				},
			})
			.catch(() => null);
		if (!record) {
			setError("エピソードの作成に失敗しました");
			return;
		}
		// ジョブの投入に失敗してもエピソードはできているので、画面へ移動して再実行できるようにする
		await enqueue
			.mutateAsync({
				type: "generateScript",
				episodeId: record.id,
				prompt,
				generateAudio,
			})
			.catch(() => undefined);
		navigate({
			to: "/podcasts/$podcastId/episodes/$episodeId",
			params: { podcastId, episodeId: record.id },
		});
	};

	if (podcasts.length === 0) {
		return (
			<p className="text-sm text-gray-500">番組を作るとエピソードを作れます</p>
		);
	}

	return (
		<form onSubmit={handleSubmit} className="flex flex-col gap-2">
			<select
				className="border rounded px-2 py-1"
				value={podcastId}
				onChange={(e) => setPodcastId(e.target.value)}
				required
			>
				{podcasts.map((p) => (
					<option key={p.id} value={p.id}>
						{p.icon} {p.title}
					</option>
				))}
			</select>
			<Input
				placeholder="タイトル"
				value={title}
				onChange={(e) => setTitle(e.target.value)}
				required
			/>
			<Textarea
				placeholder="台本生成のプロンプト"
				value={prompt}
				onChange={(e) => setPrompt(e.target.value)}
				required
			/>
			<label className="flex items-center gap-2 text-sm">
				<input
					type="checkbox"
					checked={generateAudio}
					onChange={(e) => setGenerateAudio(e.target.checked)}
				/>
				台本に続けて音声も生成する
			</label>
			<div className="flex items-center gap-2">
				<Button type="submit" disabled={pending || podcastId === ""}>
					作成して台本を生成
				</Button>
				{error && <span className="text-sm text-red-500">{error}</span>}
			</div>
		</form>
	);
}
