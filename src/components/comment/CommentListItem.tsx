import type { Comment, User } from "@prisma/client";
import { UserIcon } from "../user/UserIcon";

interface CommentListItemProps {
    user: User
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
                    <span className="text-primary font-bold">{props.user.name}</span>
                    <p className="m-0 pt-1 whitespace-pre-wrap">{props.comment.content}</p>
                </div>
            </div>
        </>
    )
}
