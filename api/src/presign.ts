// @ts-ignore: cannot resolve deps from npm package
import { S3Client } from "@bradenmacdonald/s3-lite-client";

export const s3 = new S3Client({
  endPoint: `${Deno.env.get(
    "CLOUDFLARE_ACCOUNT_ID",
  )!}.r2.cloudflarestorage.com`,
  region: "auto",
  accessKey: Deno.env.get("AWS_ACCESS_KEY_ID")!,
  secretKey: Deno.env.get("AWS_SECRET_ACCESS_KEY")!,
  bucket: "botcast",
});
