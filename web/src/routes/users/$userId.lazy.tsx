import { ChangePasswordForm } from "@/components/user/ChangePasswordForm";
import { useSession } from "@/hooks/useSession";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/users/$userId")({
	component: User,
});

function User() {
	const { userId } = Route.useParams();
	const session = useSession();
	const isMe = session.userId === userId;

	return (
		<div className="flex flex-col gap-6 p-4">
			<h2 className="text-xl font-bold">ユーザー設定</h2>
			<p className="text-sm text-gray-500">{userId}</p>
			{isMe ? (
				<ChangePasswordForm onChanged={session.saveToken} />
			) : (
				<p className="text-sm">他のユーザーの設定は変更できません</p>
			)}
		</div>
	);
}
