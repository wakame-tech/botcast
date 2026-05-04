import "@typespec/http";

using Http;

@service(#{ title: "Botcast CMS" })
@server("https://localhost:3002", "Localhost Server")
namespace BotcastCmsService;

model Collection {
  @visibility(Lifecycle.Read)
  id: string;

  name: string;
  schema: Record<string>;

  @visibility(Lifecycle.Read)
  created_at: string;
}

model CreateOrUpdateCollection is CreateOrUpdate<Collection>;

model UpdateCollection {
  name: string;
}

model Record_ {
  @visibility(Lifecycle.Read)
  id: string;

  data: Record<{}>;

  @visibility(Lifecycle.Read)
  created_at: string | null;

  @visibility(Lifecycle.Read)
  updated_at: string | null;
}

model CreateOrUpdateRecord_ is CreateOrUpdate<Record_>;

model GenerateRecordRequest {
  prompt: string;
}

// Record operation models
model CopyRecordRequest {
  /** Target collection ID to copy the record to */
  target_collection_id: string;
}

model MergeRecordsRequest {
  /** Record IDs to merge (first one becomes the target) */
  source_record_ids: string[];

  /** Target record ID (must be in source_record_ids) */
  target_record_id: string;

  /** Field names to add (numeric values will be summed) */
  add_fields?: string[];

  /** Whether to delete source records after merge (default: false) */
  delete_sources?: boolean;
}

model SplitRecordRequest {
  /** Split specifications: each object contains field names and values to subtract */
  splits: Record<{}>[];
}

model SplitRecordResponse {
  /** Original record after split (with subtracted values) */
  original: Record_;

  /** New records created from splits */
  splits: Record_[];
}

// Image field support
model ImageField {
  /** File pointer (e.g. "images:/collection_id/record_id/field_name") */
  pointer: string;

  /** MIME type (e.g. "image/png", "image/jpeg") */
  content_type: string;

  /** File size in bytes */
  size: integer;

  /** Image URL */
  url: string;
}

model UploadImageRequest {
  /** Base64 encoded image data */
  data: string;

  /** MIME type */
  content_type: "image/png" | "image/jpeg" | "image/webp" | "image/gif";

  /** Optional filename */
  filename?: string;
}

model UploadImageResponse {
  /** Saved file pointer */
  pointer: string;

  /** Image URL */
  url: string;
}


model GetImageResponse {
  /** Base64 encoded image data */
  data: string;

  /** MIME type */
  content_type: string;

  /** File size in bytes */
  size: integer;
}

@example(#{ code: "NOT_FOUND", message: "Resource not found" })
@error
model NotFoundError {
  code: "NOT_FOUND";
  message: string;
}

@example(#{ code: "VALIDATION_ERROR", message: "Invalid input data" })
@error
model ValidationError {
  code: "VALIDATION_ERROR";
  message: string;
}

@route("/collections")
@tag("Collections")
interface CollectionApi {
  @get list(): Collection[];
  @get read(@path collectionId: string): Collection;
  @post create(@body body: CreateOrUpdateCollection): {
    @statusCode statusCode: 201;
    @body body: Collection;
  };
  @put update(
    @path collectionId: string,
    @body body: UpdateCollection,
  ): Collection;
  @delete delete(@path collectionId: string): {
    @statusCode statusCode: 204;
  };
}

@route("/records")
@tag("Record")
interface RecordApi {
  @get list(@path collectionId: string): Record_[];
  @get read(@path collectionId: string, @path recordId: string): {
    @statusCode statusCode: 200;
    @body body: Record_;
  };
  @post create(@path collectionId: string, @body body: CreateOrUpdateRecord_): {
    @statusCode statusCode: 201;
    @body body: Record_;
  };
  @put update(
    @path collectionId: string,
    @path recordId: string,
    @body body: CreateOrUpdateRecord_,
  ): {
    @statusCode statusCode: 200;
    @body body: Record_;
  };
  @delete delete(@path collectionId: string, @path recordId: string): {
    @statusCode statusCode: 204;
  };
  @post @route("/{collectionId}/generate") generate(
    @path collectionId: string,
    @body body: GenerateRecordRequest,
  ): {
    @statusCode statusCode: 201;
    @body body: Record_;
  };

  @post @route("/{collectionId}/{recordId}/images/{fieldName}") uploadImage(
    @path collectionId: string,
    @path recordId: string,
    @path fieldName: string,
    @body body: UploadImageRequest,
  ): {
    @statusCode statusCode: 201;
    @body body: UploadImageResponse;
  };

  @get @route("/{collectionId}/{recordId}/images/{fieldName}") getImage(
    @path collectionId: string,
    @path recordId: string,
    @path fieldName: string,
  ): {
    @statusCode statusCode: 200;
    @header contentType: string;
    @body body: bytes;
  };

  @delete @route("/{collectionId}/{recordId}/images/{fieldName}") deleteImage(
    @path collectionId: string,
    @path recordId: string,
    @path fieldName: string,
  ): {
    @statusCode statusCode: 204;
  };

  // Record operations: copy, merge, split
  @post @route("/{collectionId}/{recordId}/copy") copyRecord(
    @path collectionId: string,
    @path recordId: string,
    @body body: CopyRecordRequest,
  ): {
    @statusCode statusCode: 201;
    @body body: Record_;
  };

  @post @route("/{collectionId}/merge") mergeRecords(
    @path collectionId: string,
    @body body: MergeRecordsRequest,
  ): {
    @statusCode statusCode: 200;
    @body body: Record_;
  };

  @post @route("/{collectionId}/{recordId}/split") splitRecord(
    @path collectionId: string,
    @path recordId: string,
    @body body: SplitRecordRequest,
  ): {
    @statusCode statusCode: 200;
    @body body: SplitRecordResponse;
  };
}

model ExecuteScriptRequest {
  language: "python3" | "nodejs";

  // The code to execute, the result must output to *stdout*.
  code: string;

  preload?: string;

  // Whether to enable network access
  enable_network?: boolean;
}

model ExecuteScriptResponse {
  code: integer;
  message: "success" | string;
  data: {
    stdout: string;
    error: string;
  };
}

@route("/scripts")
@tag("Script")
interface ScriptApi {
  @post execute(@body body: ExecuteScriptRequest): {
    @statusCode statusCode: 200;
    @body body: ExecuteScriptResponse;
  };
}

model Job {
  name: string;
  params: Record<{}>;

  @visibility(Lifecycle.Read)
  status: string;
}

alias CreateOrUpdateJob = CreateOrUpdate<Job>;

@route("/jobs")
@tag("Job")
interface JobApi {
  @post create(@body body: CreateOrUpdateJob): {
    @statusCode statusCode: 204;
  };

  @get list(): {
    @statusCode statusCode: 200;
    @body body: Job[];
  };
}

// Canvas: Graph edges between records
model Edge {
  @visibility(Lifecycle.Read)
  id: string;

  /** Source record ID in "table:id" format */
  from: string;

  /** Target record ID in "table:id" format */
  to: string;

  /** Edge label (e.g. "belongs_to", "related_to") */
  label: string;

  @visibility(Lifecycle.Read)
  created_at: string;
}

model CreateEdgeRequest {
  from: string;
  to: string;
  label: string;
}

model CanvasQueryRequest {
  /** Starting record ID in "table:id" format */
  record_id: string;

  /** Graph traversal depth (default: 1) */
  depth?: integer;

  /** Filter conditions on record fields */
  conditions?: Record<{}>;

  /** Export format */
  format: "json-canvas" | "markdown";
}

model JsonCanvasNode {
  id: string;
  x: integer;
  y: integer;
  width: integer;
  height: integer;
  type: "text";
  text: string;
}

model JsonCanvasEdge {
  id: string;
  fromNode: string;
  toNode: string;
  label?: string;
}

model JsonCanvasResponse {
  nodes: JsonCanvasNode[];
  edges: JsonCanvasEdge[];
}

model CanvasQueryResponse {
  format: "json-canvas" | "markdown";
  json_canvas?: JsonCanvasResponse;
  markdown?: string;
}

@route("/canvas")
@tag("Canvas")
interface CanvasApi {
  @route("/edges")
  @post
  createEdge(@body body: CreateEdgeRequest): {
    @statusCode statusCode: 201;
    @body body: Edge;
  };

  @route("/edges")
  @get
  listEdges(@query record_id: string): Edge[];

  @route("/edges/{edgeId}")
  @delete
  deleteEdge(@path edgeId: string): {
    @statusCode statusCode: 204;
  };

  @route("/query")
  @post
  query(@body body: CanvasQueryRequest): {
    @statusCode statusCode: 200;
    @body body: CanvasQueryResponse;
  };
}

