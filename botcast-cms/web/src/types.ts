export interface Collection {
  id: string
  name: string
  schema: Record<string, unknown>
  created_at: string
}

export interface Record_ {
  id: string
  data: Record<string, unknown>
  created_at: string | null
  updated_at: string | null
}

export interface CreateOrUpdateCollection {
  name: string
  schema: Record<string, unknown>
}

export interface CreateOrUpdateRecord {
  data: Record<string, unknown>
}

export interface GenerateRecordRequest {
  prompt: string
}

export interface ExecuteScriptRequest {
  language: 'python3' | 'nodejs'
  code: string
  preload?: string
  enable_network?: boolean
}

export interface ExecuteScriptResponse {
  code: number
  message: string
  data: {
    stdout: string
    error: string
  }
}

export interface Job {
  name: string
  params: Record<string, unknown>
  status: string
}

export interface CreateOrUpdateJob {
  name: string
  params: Record<string, unknown>
}

// Canvas
export interface Edge {
  id: string
  from: string
  to: string
  label: string
  created_at: string
}

export interface CreateEdgeRequest {
  from: string
  to: string
  label: string
}

export interface CanvasQueryRequest {
  record_id: string
  depth?: number
  conditions?: Record<string, unknown>
  format: 'json-canvas' | 'markdown'
}

export interface JsonCanvasNode {
  id: string
  x: number
  y: number
  width: number
  height: number
  type: 'text'
  text: string
}

export interface JsonCanvasEdge {
  id: string
  fromNode: string
  toNode: string
  label?: string
}

export interface JsonCanvasResponse {
  nodes: JsonCanvasNode[]
  edges: JsonCanvasEdge[]
}

export interface CanvasQueryResponse {
  format: 'json-canvas' | 'markdown'
  json_canvas?: JsonCanvasResponse
  markdown?: string
}

// Record operations
export interface CopyRecordRequest {
  target_collection_id: string
}

export interface MergeRecordsRequest {
  source_record_ids: string[]
  target_record_id: string
  add_fields?: string[]
  delete_sources: boolean
}

export interface SplitRecordRequest {
  splits: Record<string, unknown>[]
}

export interface SplitRecordResponse {
  original: Record_
  new_records: Record_[]
}

// Images
export interface UploadImageRequest {
  data: string
  content_type: string
}

export interface ImageInfo {
  pointer: string
  content_type: string
  size: number
  url: string
}

export interface GetImageResponse {
  data: string
  content_type: string
  size: number
}

// Expanded relations
export interface RelatedRecord {
  edge_id: string
  label: string
  direction: 'outgoing' | 'incoming'
  record: Record_
}

export interface RecordWithRelations {
  id: string
  data: Record<string, unknown>
  created_at: string | null
  updated_at: string | null
  relations: RelatedRecord[]
  images?: ImageInfo[]
}
