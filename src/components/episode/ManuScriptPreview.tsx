type Section = {
    type: "Serif";
    speaker: string;
    text: string;
};

export interface Manuscript {
    title: string;
    sections: Section[];
}

export function ManuscriptPreview({ manuscript }: { manuscript: Manuscript }) {
    return (
        <div>
            <p className="font-bold text-lg">title: {manuscript.title}</p>
            {manuscript.sections.map((section) => (
                <p key={section.text}>
                    <span className="pr-2 text-small text-gray">{section.speaker}</span>
                    <span>{section.text}</span>
                </p>
            ))}
        </div>
    );
}
