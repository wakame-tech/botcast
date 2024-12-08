import { Button } from "@/components/ui/button";
import { Form, FormControl, FormField, FormItem } from "@/components/ui/form";
import { Textarea } from "@/components/ui/textarea";
import { CommentInputSchema } from "@/trpc.ts";
import type { CommentInput } from "@/trpc.ts";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";

interface CommentFormProps {
	onSubmit: (values: CommentInput) => void;
}

export function CommentForm(props: CommentFormProps) {
	const form = useForm<CommentInput>({
		resolver: zodResolver(CommentInputSchema),
		defaultValues: {
			content: "",
		} satisfies CommentInput,
	});

	const onSubmit = (values: CommentInput) => {
		form.reset();
		props.onSubmit(values);
	};

	return (
		<Form {...form}>
			<form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
				<FormField
					control={form.control}
					name="content"
					render={({ field }) => (
						<FormItem>
							<FormControl>
								<Textarea rows={2} placeholder="コメントする" {...field} />
							</FormControl>
						</FormItem>
					)}
				/>
				<Button type="submit">コメント</Button>
			</form>
		</Form>
	);
}
