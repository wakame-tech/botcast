import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { useSession } from "@/hooks/useSession";
import { $cms, recordToMail, toRecordData } from "@/lib/cms_client";
import { useState } from "react";

interface EpisodeMailsProps {
	podcastId: string;
	episodeId: string;
}

/** エピソード宛てのお便り一覧と投稿フォーム */
export function EpisodeMails({ podcastId, episodeId }: EpisodeMailsProps) {
	const { userId } = useSession();
	const [radioName, setRadioName] = useState("");
	const [body, setBody] = useState("");
	const { data: records, refetch } = $cms.useQuery(
		"get",
		"/records/{collectionId}",
		{
			params: {
				path: { collectionId: "mails" },
				query: { filter: [`episode_id:eq:${episodeId}`] },
			},
		},
	);
	const createMail = $cms.useMutation("post", "/records/{collectionId}");
	const mails = (records ?? []).map(recordToMail);

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		await createMail.mutateAsync({
			params: { path: { collectionId: "mails" } },
			body: {
				data: toRecordData({
					podcast_id: podcastId,
					episode_id: episodeId,
					radio_name: radioName,
					body,
					user_id: userId ?? "",
				}),
			},
		});
		setBody("");
		await refetch();
	};

	return (
		<div className="py-4">
			<h2 className="text-lg font-bold">お便り ({mails.length})</h2>
			<ul className="p-0 list-none">
				{mails.map((mail) => (
					<li key={mail.id} className="border rounded p-2 my-2">
						<p className="text-sm font-bold">{mail.radio_name}</p>
						<p className="whitespace-pre-wrap">{mail.body}</p>
					</li>
				))}
			</ul>
			<form onSubmit={handleSubmit} className="flex flex-col gap-2">
				<Input
					placeholder="ラジオネーム"
					value={radioName}
					onChange={(e) => setRadioName(e.target.value)}
					required
				/>
				<Textarea
					placeholder="この回へのお便り"
					value={body}
					onChange={(e) => setBody(e.target.value)}
					required
				/>
				<Button type="submit" disabled={createMail.isPending}>
					この回にお便りを送る
				</Button>
			</form>
		</div>
	);
}
