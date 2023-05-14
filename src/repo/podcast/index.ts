export interface IPodcastRepository {
  upload(key: string, data: ArrayBuffer): Promise<string>;
  delete(key: string): Promise<void>;
  clean(): Promise<void>;
}
