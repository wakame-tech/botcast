export interface Serif {
  speaker: string;
  text: string;
}

export interface Episode {
  title: string;
  url: string;
  serifs: Serif[];
}

export interface Series {
  title: string;
  url: string;
  episodes: Episode[];
}
