---
id: "01KTJP1XYZABCDEF123456789A"
name: "レコード内にimage fieldを持てる"
status: "stable"
---

## 関連ファイル

- `spec/main.tsp`
- `cms/crates/api/src/openapi_adapter.rs`
- `cms/crates/api/src/repository/mod.rs`
- `cms/crates/api/src/repository/surrealdb.rs`
- `cms/crates/openapi-gen/src/apis/records.rs` (Generated)
- `docker-compose.yml`

## 機能概要

レコードの JSON Schema で `"type": "image"` を定義すると、画像データを SurrealDB の BUCKET 機能（3.0+）で管理できる。画像は Base64 または multipart/form-data でアップロードし、ファイルポインタとして保存する。レコード取得時は画像フィールドを `ImageField` オブジェクト（url, content_type, size）として返却する。

## 意図

LLM エージェントが生成したコンテンツに画像を紐づけたい（例：キャラクターのアバター、生成されたダイアグラム）。外部ストレージを別途用意せず、SurrealDB の統合ファイル管理で完結させることで、運用の複雑さを抑える。

## 前提条件

- SurrealDB 3.0 以上
- `--allow-experimental files` フラグ有効
- docker-compose.yml での BUCKET フォルダ許可設定

## 主要なメンバー

### TypeSpec モデル

```typespec
// spec/main.tsp に追加

model ImageField {
  /** ファイルポインタ (例: "images:/collection_id/record_id/field_name") */
  pointer: string;

  /** MIMEタイプ (例: "image/png", "image/jpeg") */
  content_type: string;

  /** ファイルサイズ (bytes) */
  size: integer;

  /** 画像取得URL */
  url: string;
}

model UploadImageRequest {
  /** Base64エンコードされた画像データ */
  data: string;

  /** MIMEタイプ */
  content_type: "image/png" | "image/jpeg" | "image/webp" | "image/gif";

  /** ファイル名（オプション） */
  filename?: string;
}

model UploadImageResponse {
  /** 保存されたファイルポインタ */
  pointer: string;

  /** 画像取得URL */
  url: string;
}
```

### API エンドポイント

```typespec
@route("/records")
@tag("Record")
interface RecordApi {
  // 既存のエンドポイント...

  @route("/{collectionId}/{recordId}/images/{fieldName}")
  @post
  uploadImage(
    @path collectionId: string,
    @path recordId: string,
    @path fieldName: string,
    @body body: UploadImageRequest,
  ): {
    @statusCode statusCode: 201;
    @body body: UploadImageResponse;
  };

  @route("/{collectionId}/{recordId}/images/{fieldName}")
  @get
  getImage(
    @path collectionId: string,
    @path recordId: string,
    @path fieldName: string,
  ): {
    @statusCode statusCode: 200;
    @header contentType: string;
    @body body: bytes;
  };

  @route("/{collectionId}/{recordId}/images/{fieldName}")
  @delete
  deleteImage(
    @path collectionId: string,
    @path recordId: string,
    @path fieldName: string,
  ): {
    @statusCode statusCode: 204;
  };
}
```

### Repository メソッド

- `RecordRepository::upload_image()` — SurrealDB BUCKET にファイル保存
- `RecordRepository::get_image()` — ファイルポインタからバイナリ取得
- `RecordRepository::delete_image()` — ファイル削除

### SurrealDB 設定

```surql
-- BUCKET定義（起動時に実行）
DEFINE BUCKET images BACKEND "file:/data/images";
```

### docker-compose.yml 更新

```yaml
surrealdb:
  image: surrealdb/surrealdb:v3.0.0
  command: start --user root --pass root --allow-experimental files
  environment:
    - SURREAL_LOG=trace
    - SURREAL_BUCKET_FOLDER_ALLOWLIST=/data/images
  volumes:
    - surrealdb_data:/data
    - surrealdb_images:/data/images
```

## シナリオ

### 画像フィールドを持つコレクションを作成する

1. クライアントが `POST /collections` に以下の JSON Schema を送信する：

   ```json
   {
     "name": "characters",
     "schema": {
       "type": "object",
       "properties": {
         "name": { "type": "string" },
         "avatar": { "type": "image" }
       }
     }
   }
   ```

2. API がスキーマの `"type": "image"` を認識し、内部的に `ImageField` 型として扱う
3. 201 Created でコレクション情報を返す

### レコードに画像をアップロードする

1. クライアントがレコード作成後、`POST /records/{collectionId}/{recordId}/images/avatar` に画像データを送信：

   ```json
   {
     "data": "iVBORw0KGgo...(Base64)",
     "content_type": "image/png"
   }
   ```

2. API が Base64 をデコードし、SurrealDB BUCKET に保存：

   ```surql
   f"images:/{collection_id}/{record_id}/avatar".put(<bytes>decoded_data)
   ```

3. レコードの `avatar` フィールドを更新：

   ```json
   {
     "pointer": "images:/characters/abc123/avatar",
     "content_type": "image/png",
     "size": 12345,
     "url": "/records/characters/abc123/images/avatar"
   }
   ```

4. 201 Created でファイルポインタと URL を返す

### レコード取得時に画像URLが含まれる

1. クライアントが `GET /records/{collectionId}/{recordId}` を送信
2. API がレコードを取得し、`type: "image"` フィールドを検出
3. 画像フィールドを `ImageField` オブジェクトに変換して返す：

   ```json
   {
     "id": "record:abc123",
     "data": {
       "name": "キャラクター名",
       "avatar": {
         "url": "/records/characters/abc123/images/avatar",
         "content_type": "image/png",
         "size": 12345
       }
     },
     "created_at": "2025-01-01T00:00:00Z",
     "updated_at": "2025-01-01T00:00:00Z"
   }
   ```

### 画像バイナリを直接取得する

1. クライアントが `GET /records/{collectionId}/{recordId}/images/avatar` を送信
2. API が SurrealDB からファイルデータを取得：

   ```surql
   f"images:/{collection_id}/{record_id}/avatar".get()
   ```

3. 200 OK で画像バイナリを返す（Content-Type ヘッダー付き）

### レコードの画像を削除する

1. クライアントが `DELETE /records/{collectionId}/{recordId}/images/avatar` を送信
2. API がファイルを削除し、レコードの該当フィールドを null に更新
3. 204 No Content を返す

### 存在しない画像フィールドにアクセスする

1. クライアントがスキーマに定義されていないフィールドで画像アップロードを試みる
2. API が 400 Bad Request を返す：`Field 'unknown_field' is not defined as image type`

### サイズ制限を超える画像をアップロードする

1. クライアントが 10MB を超える画像をアップロードする
2. API が 413 Payload Too Large を返す

## 設計上の考慮事項

### ストレージ方式の選択

| 方式             | メリット                         | デメリット                       |
| ---------------- | -------------------------------- | -------------------------------- |
| SurrealDB BUCKET | 統合管理、トランザクション整合性 | 実験的機能、S3連携は今後         |
| 外部S3           | スケーラビリティ、CDN連携        | 別途管理、整合性担保が複雑       |
| Base64 inline    | 実装シンプル                     | データサイズ肥大、クエリ遅延     |

**採用案**: SurrealDB BUCKET を第一候補とし、将来の S3 連携に備えて抽象化層を設ける。

### サイズ制限

- 単一画像: 10MB
- レコードあたり合計: 50MB
- RocksDB 使用時は制限緩和可能（FoundationDB は 4MB 制限あり）

### セキュリティ

- Content-Type バリデーション（マジックバイト確認）
- 画像の sanitization（EXIF 除去オプション）
- 認証済みユーザーのみアクセス可能

## 参考情報

- [File support in SurrealDB 3.0 | SurrealDB Blog](https://surrealdb.com/blog/file-support-in-surrealdb-3-0)
- [File support in SurrealDB 3.0 - DEV Community](https://dev.to/surrealdb/file-support-in-surrealdb-30-2928)
