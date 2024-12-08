import { CommentForm } from "@/components/comment/CommentForm";
import { CommentListItem } from "@/components/comment/CommentListItem";
import { trpc } from "@/trpc";
import type { CommentInput } from "@/trpc";

interface EpisodeCommentsProps {
	episodeId: string;
}

export function EpisodeComments(props: EpisodeCommentsProps) {
	const getEpisodeComments = trpc.episodeComments.useQuery({
		episodeId: props.episodeId,
	});
	const newComment = trpc.newComment.useMutation();

	const handleOnSubmitComment = async (values: CommentInput) => {
		await newComment.mutateAsync({
			episodeId: props.episodeId,
			content: values.content,
		});
		await getEpisodeComments.refetch();
	};

	const comments = getEpisodeComments.data?.comments;
	if (!comments) {
		return null;
	}

	return (
		<div className="p-2">
			{comments.map((comment) => (
				<div key={comment.id}>
					<CommentListItem user={comment.user} comment={comment} />
				</div>
			))}

			<div className="p-2">
				<CommentForm onSubmit={handleOnSubmitComment} />
			</div>
		</div>
	);
}
