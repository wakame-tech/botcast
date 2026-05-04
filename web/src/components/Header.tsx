import { Button } from "@/components/ui/button";
import { UserIcon } from "@/components/user/UserIcon";
import { useSession } from "@/hooks/useSession";
import { Link, useNavigate } from "@tanstack/react-router";

export function Header({
	isSignedIn,
	userId,
}: { isSignedIn: boolean; userId?: string }) {
	const { removeToken } = useSession();
	const navigate = useNavigate();

	const signOut = () => {
		removeToken();
		navigate({ to: "/signin" });
	};

	return (
		<>
			<div className="p-2 flex items-baseline gap-2">
				<Link to="/" className="font-bold text-teal-700 text-3xl no-underline">
					Botcast
				</Link>
				{isSignedIn && (
					<>
						<div className="pl-2">
							<Link to="/podcasts" className="no-underline">
								<span className="text-lg font-bold">ポッドキャスト</span>
							</Link>
						</div>
						<div className="pl-2">
							<Link to="/scripts" className="no-underline">
								<span className="text-lg font-bold">スクリプト</span>
							</Link>
						</div>
					</>
				)}

				<div className="flex-grow" />
				<>
					{isSignedIn && (
						<>
							{userId && <UserIcon userId={userId} />}
							<Button onClick={signOut}>サインアウト</Button>
						</>
					)}
					{!isSignedIn && (
						<Link to="/signin">
							<Button>サインイン</Button>
						</Link>
					)}
				</>
			</div>
			<hr />
		</>
	);
}
