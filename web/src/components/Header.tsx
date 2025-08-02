import { Button } from "@/components/ui/button";
import { UserIcon } from "@/components/user/UserIcon";
import { supabase } from "@/supabase";
import type { Session } from "@supabase/supabase-js";
import { Link } from "@tanstack/react-router";

export function Header({ session }: { session: Session | null }) {
	const signOut = async () => {
		await supabase.auth.signOut();
	};

	return (
		<>
			<div className="p-2 flex items-baseline gap-2">
				<Link to="/" className="font-bold text-teal-700 text-3xl no-underline">
					Botcast
				</Link>
				{session && (
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
						<div className="pl-2">
							<Link to="/tasks" className="no-underline">
								<span className="text-lg font-bold">タスク</span>
							</Link>
						</div>
					</>
				)}

				<div className="flex-grow" />
				<>
					{session && (
						<>
							<UserIcon userId={session.user.id} />
							<Button onClick={() => signOut()}>サインアウト</Button>
						</>
					)}
					{!session && (
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
