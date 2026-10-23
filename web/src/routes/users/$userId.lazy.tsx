import { NewEpisodeForm } from "@/components/episode/NewEpisodeForm";
import Podcast from "@/components/podcast/PodcastList";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { UserSettingsForm } from "@/components/user/UserSettingsForm";
import { useSession } from "@/hooks/useSession";
import { $cms, CMS_TENANT, recordToPodcast } from "@/lib/cms_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/users/$userId")({
	component: User,
});

function User() {
	const { userId } = Route.useParams();
	const session = useSession();
	const { data: records } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}",
		{
			params: { path: { tenantId: CMS_TENANT, collectionId: "podcasts" } },
		},
	);
	// CMS の filter は data のフィールドが対象なので、所有者はここで絞る
	const podcasts = (records ?? [])
		.map(recordToPodcast)
		.filter((p) => p.owner_id === userId);
	const isMe = session.userId === userId;

	return (
		<div className="flex flex-col gap-6 p-4">
			<div>
				<h2 className="text-xl font-bold">ユーザー</h2>
				<p className="text-sm text-gray-500">{userId}</p>
			</div>

			{isMe && (
				<Card>
					<CardHeader>
						<CardTitle>新しいエピソード</CardTitle>
					</CardHeader>
					<CardContent>
						{records && <NewEpisodeForm podcasts={podcasts} />}
					</CardContent>
				</Card>
			)}

			{isMe && (
				<Card>
					<CardHeader>
						<CardTitle>設定</CardTitle>
					</CardHeader>
					<CardContent>
						<UserSettingsForm />
					</CardContent>
				</Card>
			)}

			<div>
				<h2 className="text-lg font-bold">番組 ({podcasts.length})</h2>
				<Podcast.List podcasts={podcasts} />
			</div>
		</div>
	);
}
