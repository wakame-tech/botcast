import { Button } from "@/components/ui/button";
import {
    Form,
    FormControl,
    FormField,
    FormItem,
} from "@/components/ui/form";
import { Textarea } from "@/components/ui/textarea";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";

interface CommentFormProps {
    onSubmit: (values: CommentEditFormValues) => void;
}

const commentEditFormSchema = z.object({
    content: z.string(),
});

export type CommentEditFormValues = z.infer<typeof commentEditFormSchema>;

export function CommentForm(props: CommentFormProps) {
    const form = useForm<z.infer<typeof commentEditFormSchema>>({
        resolver: zodResolver(commentEditFormSchema),
        defaultValues: {
            content: '',
        },
    });

    const onSubmit = (values: CommentEditFormValues) => {
        form.reset();
        props.onSubmit(values);
    }

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
                <Button type="submit">
                    コメント
                </Button>
            </form>
        </Form>
    );
}
