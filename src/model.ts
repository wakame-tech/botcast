export interface Script {
  id: string;
  title: string;
  scenes: Scene[];
}

export interface Feed {
  id: string;
  title: string;
  description: string;
  date: Date;
  url: string;
}

export type Scene = Serif;
// | Sound;

export interface Serif {
  type: "serif";
  id: string;
  speaker: string;
  text: string;
}

export interface Sound {
  type: "sound";
  id: string;
  path: string;
}
