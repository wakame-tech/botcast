import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { SecretList } from "@/components/user/SecretsList";
import type { NewSecret } from "@/components/user/SecretsList";
import { $api } from "@/lib/api_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/users/$userId")({
	component: User,
});

function User() {
	const secretsQuery = $api.useQuery("get", "/secrets");
	const updateSecrets = $api.useMutation("post", "/secrets");

	const onSubmit = async (news: NewSecret[], deletionIds: string[]) => {
		await updateSecrets.mutateAsync({
			body: {
				news,
				deletionIds,
			},
		});
		secretsQuery.refetch();
	};

	if (!secretsQuery.data) {
		return null;
	}

	const secrets = secretsQuery.data;

	return (
		<>
			<Card>
				<CardTitle>環境変数</CardTitle>
				<CardContent>
					<SecretList secrets={secrets} onSubmit={onSubmit} />
				</CardContent>
			</Card>
		</>
	);
}
