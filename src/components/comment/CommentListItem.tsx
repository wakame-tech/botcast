import type { Comment } from "@/trpc";
import type { User } from "@prisma/client";
import { UserIcon } from "../user/UserIcon";

interface CommentListItemProps {
	user: User;
	comment: Comment;
}

export function CommentListItem(props: CommentListItemProps) {
	return (
		<>
			<div className="flex gap-2">
				<div>
					<UserIcon userId={props.user.id} size="2rem" />
				</div>
				<div>
					<span className="font-bold">{props.user.name}</span>
					<span className="text-xs text-gray">
						{new Date(props.comment.created_at).toLocaleString()}
					</span>
					<span className="pl-2 m-0 pt-1 whitespace-pre-wrap">
						{props.comment.content}
					</span>
				</div>
			</div>
		</>
	);
}
