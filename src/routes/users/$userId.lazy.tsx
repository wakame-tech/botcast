import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { SecretList } from "@/components/user/SecretsList";
import type { NewSecret } from "@/components/user/SecretsList";
import { trpc } from "@/trpc.ts";
import { Link, createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/users/$userId")({
	component: User,
});

function User() {
	const secretsQuery = trpc.secrets.useQuery();
	const updateSecrets = trpc.updateSecrets.useMutation();

	const onSubmit = async (news: NewSecret[], deletionIds: string[]) => {
		await updateSecrets.mutateAsync({ news, deletionIds });
		secretsQuery.refetch();
	};

	if (!secretsQuery.data) {
		return null;
	}

	const secrets = secretsQuery.data.secrets;

	return (
		<>
			<Card>
				<CardTitle>デバッグ</CardTitle>
				<CardContent>
					<Link to="/tasks">タスク</Link>
					<Link to="/scripts">スクリプト</Link>
				</CardContent>

				<CardTitle>環境変数</CardTitle>
				<CardContent>
					<SecretList secrets={secrets} onSubmit={onSubmit} />
				</CardContent>
			</Card>
		</>
	);
}
