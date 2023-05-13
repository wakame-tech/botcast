export interface IPodcastRepository {
  upload(key: string, data: ArrayBuffer): Promise<string>;
}
