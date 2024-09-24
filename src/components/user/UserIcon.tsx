import { Link } from "@tanstack/react-router";
import Avatar from "boring-avatars";

interface UserIconProps {
	size?: string;
	userId: string;
	label?: string;
}

export function UserIcon(props: UserIconProps) {
	return (
		<>
			<span className="my-auto">
				<Link
					to="/users/$userId"
					params={{ userId: props.userId }}
					className="flex items-center no-underline"
				>
					<Avatar
						name={props.userId}
						colors={["#009688", "#ffffff"]}
						variant="beam"
						size={props.size ?? "2.5rem"}
					/>
					{props.label && (
						<span className="pl-2 text-primary text-lg font-bold">
							{props.label}
						</span>
					)}
				</Link>
			</span>
		</>
	);
}
