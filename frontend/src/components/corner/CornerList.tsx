import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface Corner {
	id: string;
	title: string;
	description?: string | null;
	requesting_mail?: boolean;
	cms_collection_id?: string | null;
}

interface CornerListProps {
	corners: Corner[];
}

export function CornerList(props: CornerListProps) {
	return (
		<div className="flex flex-wrap gap-4">
			{props.corners.map((corner) => (
				<div key={corner.id}>
					<CornerListItem corner={corner} />
				</div>
			))}
		</div>
	);
}

interface CornerListItemProps {
	corner: Corner;
}

function CornerListItem(props: CornerListItemProps) {
	return (
		<>
			<Card className="bg-gray-200">
				<CardHeader className="py-0">
					<CardTitle>
						<div className="flex-inline items-center gap-2">
							<span className="text-xl font-bold">{props.corner.title}</span>
						</div>
					</CardTitle>
				</CardHeader>
				<CardContent>{props.corner.description}</CardContent>
			</Card>
		</>
	);
}
