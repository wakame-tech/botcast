import type { User } from "@prisma/client";

interface UserIconProps {
	user: User;
}

export function UserIcon(props: UserIconProps) {
	// TODO: use user's avatar
	return (
		<div className="flex-inline justify-center items-center w-8 h-8 rounded-full bg-teal-700">
			<span className="text-white text-xl font-bold">
				{(props.user.name ?? "?")[0]}
			</span>
		</div>
	);
}
