import { Button } from "@/components/ui/button";
import {
    Form,
    FormControl,
    FormField,
    FormItem,
    FormLabel,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";

interface ScriptFormProps {
    disabled?: boolean;
    template: string;
    onSubmit: (values: ScriptEditFormValues) => void;
}

const scriptEditFormSchema = z.object({
    title: z.string(),
    template: z.string(),
});

export type ScriptEditFormValues = z.infer<typeof scriptEditFormSchema>;

export function ScriptForm(props: ScriptFormProps) {
    const form = useForm<z.infer<typeof scriptEditFormSchema>>({
        resolver: zodResolver(scriptEditFormSchema),
        defaultValues: {
            title: '',
            template: props.template,
        },
    });

    return (
        <Form {...form}>
            <form onSubmit={form.handleSubmit(props.onSubmit)} className="space-y-8">
                <FormField
                    control={form.control}
                    name="title"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>タイトル</FormLabel>
                            <FormControl>
                                <Input placeholder="title" {...field} />
                            </FormControl>
                        </FormItem>
                    )}
                />
                <FormField
                    control={form.control}
                    name="template"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>スクリプト</FormLabel>
                            <FormControl>
                                <Textarea placeholder="template" {...field} />
                            </FormControl>
                        </FormItem>
                    )}
                />
                <Button disabled={props.disabled} type="submit">
                    作成
                </Button>
            </form>
        </Form>
    );
}
