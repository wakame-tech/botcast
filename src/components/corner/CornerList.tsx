import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { Corner } from "@/lib/api_client";
import { Link } from "@tanstack/react-router";
import { Button } from "../ui/button";

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
				<CardContent>
					{props.corner.description}
					{props.corner.requesting_mail && (
						<div>
							<Link
								to="/corners/$cornerId/newMail"
								params={{ cornerId: props.corner.id }}
							>
								<Button>メールを送る</Button>
							</Link>

							<Link
								to="/corners/$cornerId"
								params={{ cornerId: props.corner.id }}
							>
								<Button>コーナー詳細</Button>
							</Link>
						</div>
					)}
				</CardContent>
			</Card>
		</>
	);
}
