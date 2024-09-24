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
			<div className="p-2 flex justify-items-center gap-2">
				<Link to="/" className="font-bold text-teal-700 text-3xl no-underline">
					Botcast
				</Link>
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
