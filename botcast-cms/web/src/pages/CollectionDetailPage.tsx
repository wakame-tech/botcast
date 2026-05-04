// @specre 01KNWXPQ69MWA8ZAK2PXCYH4SE
// @specre 01KNWXPMN9DYWM9NQRYKHRCGHW
// @specre 01KNWXPJMJ8J5FVHQWXDKQBTX6
// @specre 01KNWXPEQR0JX0VJC8JFME6JF3
// @specre 01KNM2BBT5T43CMWAX32H3K14G
// @specre 01KNM91QXA84F7HTBYSKM5TZMP
// @specre 01KNM91QXWGPMAEZZE0M7GCTB4
// @specre 01KNVEV4DB8QKB8DYKJQHYKDQD
import { useEffect, useState, useCallback } from 'react'
import { useParams, Link } from 'react-router-dom'
import {
  collectionsApi,
  recordsApi,
  canvasApi,
  imagesApi,
  rawId,
} from '../api/client'
import type {
  Collection,
  Record_,
  RecordWithRelations,
  RelatedRecord,
  Edge,
  CanvasQueryResponse,
  JsonCanvasNode,
  JsonCanvasEdge,
} from '../types'
import { generateDummyData } from '../utils/generateDummyData'
import './Page.css'

/** Extract image field names from JSON Schema
 * Detects fields with format: "image" or type: "image" (custom extension)
 */
function getImageFields(schema: Record<string, unknown>): string[] {
  const props = schema.properties as Record<string, { type?: string; format?: string }> | undefined
  if (!props) return []
  return Object.entries(props)
    .filter(([, v]) => v.format === 'image' || v.type === 'image')
    .map(([k]) => k)
}

function formatDate(s: string | null) {
  if (!s) return '-'
  try {
    return new Date(s).toLocaleString('ja-JP')
  } catch {
    return s
  }
}

/** Check if value is an image info object */
function isImageInfo(val: unknown): val is { url: string; content_type: string; pointer: string } {
  return (
    typeof val === 'object' &&
    val !== null &&
    'url' in val &&
    'content_type' in val &&
    typeof (val as { url: unknown }).url === 'string' &&
    (val as { url: string }).url.includes('/images/')
  )
}

/** Component to display an image from API */
function ImageFromApi({ url }: { url: string }) {
  const [dataUrl, setDataUrl] = useState<string | null>(null)
  const [error, setError] = useState(false)

  useEffect(() => {
    // Parse URL: /records/{collectionId}/{recordId}/images/{fieldName}
    const match = url.match(/\/records\/([^/]+)\/([^/]+)\/images\/([^/]+)/)
    if (!match) {
      setError(true)
      return
    }
    const [, collectionId, recordId, fieldName] = match

    imagesApi.get(collectionId, recordId, fieldName)
      .then((res) => {
        setDataUrl(`data:${res.content_type};base64,${res.data}`)
      })
      .catch(() => setError(true))
  }, [url])

  if (error) return <span className='muted'>画像読込失敗</span>
  if (!dataUrl) return <span className='muted'>読込中...</span>
  return <img src={dataUrl} alt='' className='record-field-image' />
}

/** Render a record data object as a key-value field list */
function RecordFields({ data }: { data: Record<string, unknown> }) {
  const entries = Object.entries(data)
  if (entries.length === 0) return <span className='muted'>（空）</span>
  return (
    <dl className='record-fields'>
      {entries.map(([key, val]) => (
        <div key={key} className='record-field'>
          <dt className='record-field-key'>{key}</dt>
          <dd className='record-field-val'>
            {isImageInfo(val) ? (
              <ImageFromApi url={val.url} />
            ) : typeof val === 'object' && val !== null ? (
              JSON.stringify(val, null, 2)
            ) : (
              String(val ?? '')
            )}
          </dd>
        </div>
      ))}
    </dl>
  )
}

/** Show an edge as an arrow card with related record fields */
function EdgeCard({
  edge,
  selfId,
  relatedRecord,
  onDelete,
}: {
  edge: Edge
  selfId: string
  relatedRecord?: Record_
  onDelete: () => void
}) {
  const isOutgoing = edge.from === selfId
  const other = isOutgoing ? edge.to : edge.from
  return (
    <div className='edge-card'>
      <div className='edge-card-header'>
        <span className='edge-card-dir'>{isOutgoing ? '→' : '←'}</span>
        <span className='edge-label-badge'>{edge.label}</span>
        <span className='edge-card-target mono-text'>{other}</span>
        <button
          className='btn btn-danger btn-sm edge-card-del'
          onClick={onDelete}
        >
          削除
        </button>
      </div>
      {relatedRecord && (
        <div className='edge-card-body'>
          <RecordFields data={relatedRecord.data} />
        </div>
      )}
    </div>
  )
}

/** Show a pre-loaded relation inline */
function RelationCard({ relation }: { relation: RelatedRecord }) {
  return (
    <div className='edge-card'>
      <div className='edge-card-header'>
        <span className='edge-card-dir'>
          {relation.direction === 'outgoing' ? '→' : '←'}
        </span>
        <span className='edge-label-badge'>{relation.label}</span>
        <span className='edge-card-target mono-text'>
          {relation.record.id}
        </span>
      </div>
      <div className='edge-card-body'>
        <RecordFields data={relation.record.data} />
      </div>
    </div>
  )
}

/** Visual graph from JSON Canvas data */
function CanvasGraph({
  nodes,
  edges,
}: {
  nodes: JsonCanvasNode[]
  edges: JsonCanvasEdge[]
}) {
  const nodeMap = new Map(nodes.map((n) => [n.id, n]))
  return (
    <div className='canvas-graph'>
      <div className='canvas-graph-nodes'>
        {nodes.map((n) => (
          <div key={n.id} className='canvas-graph-node'>
            <div className='canvas-graph-node-text'>{n.text}</div>
          </div>
        ))}
      </div>
      <div className='canvas-graph-edges'>
        {edges.map((e) => {
          const from = nodeMap.get(e.fromNode)
          const to = nodeMap.get(e.toNode)
          return (
            <div key={e.id} className='canvas-graph-edge'>
              <span className='canvas-graph-edge-from'>
                {from?.text ?? e.fromNode}
              </span>
              <span className='canvas-graph-edge-arrow'>
                —<span className='edge-label-badge'>{e.label ?? ''}</span>→
              </span>
              <span className='canvas-graph-edge-to'>
                {to?.text ?? e.toNode}
              </span>
            </div>
          )
        })}
      </div>
    </div>
  )
}

/** Simple markdown renderer for basic elements */
function RenderedMarkdown({ text }: { text: string }) {
  const html = text
    .replace(/^### (.+)$/gm, '<h3>$1</h3>')
    .replace(/^## (.+)$/gm, '<h2>$1</h2>')
    .replace(/^# (.+)$/gm, '<h1>$1</h1>')
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/^- (.+)$/gm, '<li>$1</li>')
    .replace(/(<li>.*<\/li>\n?)+/g, (m) => `<ul>${m}</ul>`)
    .replace(/\|(.+)\|/g, (m) => `<div class="md-table-row">${m}</div>`)
    .replace(/\n{2,}/g, '<br/><br/>')
    .replace(/\n/g, '<br/>')
  return (
    <div
      className='rendered-markdown'
      dangerouslySetInnerHTML={{ __html: html }}
    />
  )
}

export function CollectionDetailPage() {
  const { collectionId } = useParams<{ collectionId: string }>()

  const [collection, setCollection] = useState<Collection | null>(null)
  const [records, setRecords] = useState<RecordWithRelations[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Create form
  const [showCreate, setShowCreate] = useState(false)
  const [createData, setCreateData] = useState('{}')
  const [createError, setCreateError] = useState<string | null>(null)
  const [creating, setCreating] = useState(false)
  const [createImageFiles, setCreateImageFiles] = useState<Record<string, File>>({})
  const [createImagePreviews, setCreateImagePreviews] = useState<Record<string, string>>({})

  // Get image fields from schema
  const imageFields = collection ? getImageFields(collection.schema) : []

  // Generate form
  const [showGenerate, setShowGenerate] = useState(false)
  const [generatePrompt, setGeneratePrompt] = useState('')
  const [generating, setGenerating] = useState(false)
  const [generateError, setGenerateError] = useState<string | null>(null)

  // Edit record
  const [editId, setEditId] = useState<string | null>(null)
  const [editData, setEditData] = useState('{}')
  const [editError, setEditError] = useState<string | null>(null)

  // Canvas: edges per record
  const [edgeRecordId, setEdgeRecordId] = useState<string | null>(null)
  const [edges, setEdges] = useState<Edge[]>([])
  const [edgesLoading, setEdgesLoading] = useState(false)
  const [edgesError, setEdgesError] = useState<string | null>(null)
  const [showCreateEdge, setShowCreateEdge] = useState(false)
  const [edgeTo, setEdgeTo] = useState('')
  const [edgeLabel, setEdgeLabel] = useState('')
  const [creatingEdge, setCreatingEdge] = useState(false)
  const [createEdgeError, setCreateEdgeError] = useState<string | null>(null)

  // Copy record
  const [copyRecordId, setCopyRecordId] = useState<string | null>(null)
  const [copyTargetCollectionId, setCopyTargetCollectionId] = useState('')
  const [copying, setCopying] = useState(false)
  const [copyError, setCopyError] = useState<string | null>(null)
  const [allCollections, setAllCollections] = useState<Collection[]>([])

  // Merge records
  const [showMerge, setShowMerge] = useState(false)
  const [mergeSourceIds, setMergeSourceIds] = useState<Set<string>>(new Set())
  const [mergeTargetId, setMergeTargetId] = useState<string | null>(null)
  const [mergeAddFields, setMergeAddFields] = useState('')
  const [mergeDeleteSources, setMergeDeleteSources] = useState(false)
  const [merging, setMerging] = useState(false)
  const [mergeError, setMergeError] = useState<string | null>(null)

  // Split record
  const [splitRecordId, setSplitRecordId] = useState<string | null>(null)
  const [splitData, setSplitData] = useState('[{}]')
  const [splitting, setSplitting] = useState(false)
  const [splitError, setSplitError] = useState<string | null>(null)

  // Image management
  const [imageRecordId, setImageRecordId] = useState<string | null>(null)
  const [imageFieldName, setImageFieldName] = useState('')
  const [uploading, setUploading] = useState(false)
  const [imageError, setImageError] = useState<string | null>(null)
  const [imagePreview, setImagePreview] = useState<string | null>(null)

  // Canvas: subgraph query per record
  const [graphRecordId, setGraphRecordId] = useState<string | null>(null)
  const [queryDepth, setQueryDepth] = useState(1)
  const [queryFormat, setQueryFormat] = useState<'json-canvas' | 'markdown'>(
    'json-canvas',
  )
  const [queryResult, setQueryResult] = useState<CanvasQueryResponse | null>(
    null,
  )
  const [querying, setQuerying] = useState(false)
  const [queryError, setQueryError] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  const load = useCallback(async () => {
    if (!collectionId) return
    try {
      setLoading(true)
      const [col, recs] = await Promise.all([
        collectionsApi.read(collectionId),
        recordsApi.listWithRelations(collectionId),
      ])
      setCollection(col)
      setRecords(recs)
      setError(null)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }, [collectionId])

  useEffect(() => {
    load()
  }, [load])

  if (!collectionId) return null

  const handleCreateImageSelect = (fieldName: string, file: File | null) => {
    if (file) {
      // Store file and create preview URL
      setCreateImageFiles(prev => ({ ...prev, [fieldName]: file }))
      const previewUrl = URL.createObjectURL(file)
      setCreateImagePreviews(prev => ({ ...prev, [fieldName]: previewUrl }))
    } else {
      // Clear file and preview
      setCreateImageFiles(prev => {
        const next = { ...prev }
        delete next[fieldName]
        return next
      })
      setCreateImagePreviews(prev => {
        if (prev[fieldName]) URL.revokeObjectURL(prev[fieldName])
        const next = { ...prev }
        delete next[fieldName]
        return next
      })
    }
  }

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    setCreateError(null)
    let parsed: unknown
    try {
      parsed = JSON.parse(createData)
    } catch {
      setCreateError('Data は有効な JSON で入力してください')
      return
    }
    try {
      setCreating(true)
      // Create record first
      const created = await recordsApi.create(collectionId, {
        data: parsed as Record<string, unknown>,
      })

      // Upload images to the created record
      for (const [fieldName, file] of Object.entries(createImageFiles)) {
        const reader = new FileReader()
        const base64 = await new Promise<string>((resolve, reject) => {
          reader.onload = () => {
            const result = reader.result as string
            resolve(result.split(',')[1])
          }
          reader.onerror = reject
          reader.readAsDataURL(file)
        })
        await imagesApi.upload(collectionId, created.id, fieldName, {
          data: base64,
          content_type: file.type || 'application/octet-stream',
        })
      }

      // Clean up preview URLs
      Object.values(createImagePreviews).forEach(url => URL.revokeObjectURL(url))

      setCreateData('{}')
      setCreateImageFiles({})
      setCreateImagePreviews({})
      setShowCreate(false)
      await load()
    } catch (e) {
      setCreateError(String(e))
    } finally {
      setCreating(false)
    }
  }

  const handleGenerate = async (e: React.FormEvent) => {
    e.preventDefault()
    setGenerateError(null)
    try {
      setGenerating(true)
      await recordsApi.generate(collectionId, { prompt: generatePrompt })
      setGeneratePrompt('')
      setShowGenerate(false)
      await load()
    } catch (e) {
      setGenerateError(String(e))
    } finally {
      setGenerating(false)
    }
  }

  const handleDelete = async (recordId: string) => {
    if (!confirm('このレコードを削除しますか？')) return
    try {
      await recordsApi.delete(collectionId, recordId)
      await load()
    } catch (e) {
      alert(String(e))
    }
  }

  const startEdit = (r: Record_) => {
    setEditId(r.id)
    setEditData(JSON.stringify(r.data, null, 2))
    setEditError(null)
  }

  const handleEditSave = async (recordId: string) => {
    setEditError(null)
    let parsed: unknown
    try {
      parsed = JSON.parse(editData)
    } catch {
      setEditError('Data は有効な JSON で入力してください')
      return
    }
    try {
      await recordsApi.update(collectionId, recordId, {
        data: parsed as Record<string, unknown>,
      })
      setEditId(null)
      await load()
    } catch (e) {
      setEditError(String(e))
    }
  }

  // Copy record handlers
  const openCopy = async (recordId: string) => {
    setCopyRecordId(recordId)
    setCopyError(null)
    setCopyTargetCollectionId('')
    try {
      const cols = await collectionsApi.list()
      setAllCollections(cols.filter((c) => rawId(c.id) !== collectionId))
    } catch (e) {
      setCopyError(String(e))
    }
  }

  const handleCopyRecord = async () => {
    if (!copyRecordId || !copyTargetCollectionId) return
    setCopyError(null)
    try {
      setCopying(true)
      await recordsApi.copy(collectionId, copyRecordId, {
        target_collection_id: copyTargetCollectionId,
      })
      setCopyRecordId(null)
      await load()
    } catch (e) {
      setCopyError(String(e))
    } finally {
      setCopying(false)
    }
  }

  // Merge record handlers
  const toggleMergeSelect = (recordId: string) => {
    setMergeSourceIds((prev) => {
      const next = new Set(prev)
      if (next.has(recordId)) {
        next.delete(recordId)
        if (mergeTargetId === recordId) setMergeTargetId(null)
      } else {
        next.add(recordId)
      }
      return next
    })
  }

  const handleMerge = async () => {
    if (!mergeTargetId || mergeSourceIds.size < 2) return
    setMergeError(null)
    const addFields = mergeAddFields
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)
    try {
      setMerging(true)
      await recordsApi.merge(collectionId, {
        source_record_ids: Array.from(mergeSourceIds),
        target_record_id: mergeTargetId,
        add_fields: addFields.length > 0 ? addFields : undefined,
        delete_sources: mergeDeleteSources,
      })
      setShowMerge(false)
      setMergeSourceIds(new Set())
      setMergeTargetId(null)
      setMergeAddFields('')
      setMergeDeleteSources(false)
      await load()
    } catch (e) {
      setMergeError(String(e))
    } finally {
      setMerging(false)
    }
  }

  // Split record handlers
  const handleSplit = async (recordId: string) => {
    setSplitError(null)
    let parsed: unknown
    try {
      parsed = JSON.parse(splitData)
    } catch {
      setSplitError('Splits は有効な JSON 配列で入力してください')
      return
    }
    if (!Array.isArray(parsed) || parsed.length === 0) {
      setSplitError('Splits は空でない配列で入力してください')
      return
    }
    try {
      setSplitting(true)
      await recordsApi.split(collectionId, recordId, {
        splits: parsed as Record<string, unknown>[],
      })
      setSplitRecordId(null)
      setSplitData('[{}]')
      await load()
    } catch (e) {
      setSplitError(String(e))
    } finally {
      setSplitting(false)
    }
  }

  // Image handlers
  const handleImageUpload = async (
    recordId: string,
    file: File,
    fieldName: string,
  ) => {
    setImageError(null)
    try {
      setUploading(true)
      const reader = new FileReader()
      const base64 = await new Promise<string>((resolve, reject) => {
        reader.onload = () => {
          const result = reader.result as string
          resolve(result.split(',')[1])
        }
        reader.onerror = reject
        reader.readAsDataURL(file)
      })
      await imagesApi.upload(collectionId, recordId, fieldName, {
        data: base64,
        content_type: file.type || 'application/octet-stream',
      })
      setImageFieldName('')
      setImageRecordId(null)
      setImagePreview(null)
      await load()
    } catch (e) {
      setImageError(String(e))
    } finally {
      setUploading(false)
    }
  }

  const handleImageDelete = async (recordId: string, fieldName: string) => {
    if (!confirm(`画像 "${fieldName}" を削除しますか？`)) return
    try {
      await imagesApi.delete(collectionId, recordId, fieldName)
      await load()
    } catch (e) {
      setImageError(String(e))
    }
  }

  const handleViewImage = async (recordId: string, fieldName: string) => {
    try {
      const img = await imagesApi.get(collectionId, recordId, fieldName)
      setImagePreview(`data:${img.content_type};base64,${img.data}`)
      setImageRecordId(recordId)
    } catch (e) {
      setImageError(String(e))
    }
  }

  /** Build "table:key" record ID for Canvas API */
  const fullRecordId = (rId: string) => `${collectionId}:${rawId(rId)}`

  // Canvas: load edges for a record (for edge management)
  const toggleEdges = async (recordId: string) => {
    if (edgeRecordId === recordId) {
      setEdgeRecordId(null)
      setEdges([])
      setShowCreateEdge(false)
      return
    }
    setEdgeRecordId(recordId)
    setGraphRecordId(null)
    setShowCreateEdge(false)
    setEdgesError(null)
    try {
      setEdgesLoading(true)
      const edgeList = await canvasApi.listEdges(fullRecordId(recordId))
      setEdges(edgeList)
    } catch (e) {
      setEdgesError(String(e))
    } finally {
      setEdgesLoading(false)
    }
  }

  const handleCreateEdge = async (fromId: string) => {
    setCreateEdgeError(null)
    if (!edgeTo.trim() || !edgeLabel.trim()) {
      setCreateEdgeError('To と Label を入力してください')
      return
    }
    try {
      setCreatingEdge(true)
      await canvasApi.createEdge({
        from: fullRecordId(fromId),
        to: edgeTo.trim(),
        label: edgeLabel.trim(),
      })
      setEdgeTo('')
      setEdgeLabel('')
      setShowCreateEdge(false)
      setEdges(await canvasApi.listEdges(fullRecordId(fromId)))
      await load()
    } catch (e) {
      setCreateEdgeError(String(e))
    } finally {
      setCreatingEdge(false)
    }
  }

  const handleDeleteEdge = async (edgeId: string, recordId: string) => {
    if (!confirm('このエッジを削除しますか？')) return
    try {
      await canvasApi.deleteEdge(edgeId)
      setEdges(await canvasApi.listEdges(fullRecordId(recordId)))
      await load()
    } catch (e) {
      setEdgesError(String(e))
    }
  }

  // Canvas: subgraph query
  const toggleGraph = (recordId: string) => {
    if (graphRecordId === recordId) {
      setGraphRecordId(null)
      setQueryResult(null)
      return
    }
    setGraphRecordId(recordId)
    setEdgeRecordId(null)
    setQueryResult(null)
    setQueryError(null)
    setQueryDepth(1)
    setQueryFormat('json-canvas')
  }

  const handleQuery = async (recordId: string) => {
    setQueryError(null)
    setQueryResult(null)
    try {
      setQuerying(true)
      setQueryResult(
        await canvasApi.query({
          record_id: fullRecordId(recordId),
          depth: queryDepth,
          format: queryFormat,
        }),
      )
    } catch (e) {
      setQueryError(String(e))
    } finally {
      setQuerying(false)
    }
  }

  const handleCopy = async () => {
    if (!queryResult) return
    const text =
      queryResult.format === 'json-canvas' && queryResult.json_canvas
        ? JSON.stringify(queryResult.json_canvas, null, 2)
        : (queryResult.markdown ?? '')
    await navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className='page'>
      <div className='breadcrumb'>
        <Link to='/'>Collections</Link>
        <span className='muted'> / </span>
        <span>{collection?.name ?? collectionId}</span>
      </div>

      <div className='page-header'>
        <h1 className='page-title'>{collection?.name ?? '...'}</h1>
        <div className='row-actions'>
          <button
            className='btn btn-ghost btn-sm'
            onClick={() => {
              setShowGenerate(!showGenerate)
              setShowCreate(false)
            }}
          >
            AI 生成
          </button>
          <button
            className={`btn btn-sm ${showMerge ? 'btn-primary' : 'btn-ghost'}`}
            onClick={() => {
              setShowMerge(!showMerge)
              if (showMerge) {
                setMergeSourceIds(new Set())
                setMergeTargetId(null)
                setMergeError(null)
              }
            }}
          >
            {showMerge ? 'マージ解除' : 'マージ'}
          </button>
          <button
            className='btn btn-primary btn-sm'
            onClick={() => {
              if (!showCreate && collection?.schema) {
                const dummyData = generateDummyData(collection.schema)
                setCreateData(JSON.stringify(dummyData, null, 2))
              }
              setShowCreate(!showCreate)
              setShowGenerate(false)
            }}
          >
            + 手動追加
          </button>
        </div>
      </div>

      {showMerge && (
        <div className='card form-card'>
          <h2 className='card-title'>レコードマージ</h2>
          <p className='muted' style={{ margin: '0 0 0.75rem' }}>
            マージ対象のレコードを選択し、マージ先を指定してください（最低2件選択）
          </p>
          <div className='form-group'>
            <label className='form-label'>
              加算フィールド（カンマ区切り、数値フィールドの値を合算）
            </label>
            <input
              className='form-input'
              value={mergeAddFields}
              onChange={(e) => setMergeAddFields(e.target.value)}
              placeholder='例: count,score'
            />
          </div>
          <label className='form-checkbox-label'>
            <input
              type='checkbox'
              checked={mergeDeleteSources}
              onChange={(e) => setMergeDeleteSources(e.target.checked)}
            />
            <span>マージ元レコードを削除する</span>
          </label>
          {mergeError && <p className='error-text'>{mergeError}</p>}
          <div className='form-actions' style={{ marginTop: '0.75rem' }}>
            <button
              className='btn btn-primary btn-sm'
              disabled={merging || mergeSourceIds.size < 2 || !mergeTargetId}
              onClick={handleMerge}
            >
              {merging ? 'マージ中...' : `マージ実行 (${mergeSourceIds.size}件)`}
            </button>
          </div>
        </div>
      )}

      {collection && (
        <details className='schema-details'>
          <summary className='schema-summary'>JSON Schema を確認</summary>
          <pre className='code-block'>
            {JSON.stringify(collection.schema, null, 2)}
          </pre>
        </details>
      )}

      {showCreate && (
        <form className='card form-card' onSubmit={handleCreate}>
          <h2 className='card-title'>レコード手動追加</h2>
          <div className='form-group'>
            <label className='form-label'>Data (JSON)</label>
            <textarea
              className='form-textarea'
              value={createData}
              onChange={(e) => setCreateData(e.target.value)}
              rows={8}
              disabled={creating}
              spellCheck={false}
            />
          </div>

          {/* Image field upload buttons */}
          {imageFields.length > 0 && (
            <div className='form-group'>
              <label className='form-label'>画像フィールド</label>
              <div className='image-fields-grid'>
                {imageFields.map((fieldName) => (
                  <div key={fieldName} className='image-field-item'>
                    <span className='image-field-name'>{fieldName}</span>
                    {createImagePreviews[fieldName] ? (
                      <div className='image-field-preview'>
                        <img
                          src={createImagePreviews[fieldName]}
                          alt={fieldName}
                        />
                        <button
                          type='button'
                          className='btn btn-ghost btn-sm'
                          onClick={() => handleCreateImageSelect(fieldName, null)}
                          disabled={creating}
                        >
                          クリア
                        </button>
                      </div>
                    ) : (
                      <label className='btn btn-ghost btn-sm image-field-btn'>
                        画像を選択
                        <input
                          type='file'
                          accept='image/*'
                          style={{ display: 'none' }}
                          onChange={(e) => {
                            const file = e.target.files?.[0]
                            if (file) handleCreateImageSelect(fieldName, file)
                            e.target.value = ''
                          }}
                          disabled={creating}
                        />
                      </label>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {createError && <p className='error-text'>{createError}</p>}
          <div className='form-actions'>
            <button
              className='btn btn-primary'
              type='submit'
              disabled={creating}
            >
              {creating ? '追加中...' : '追加'}
            </button>
            <button
              className='btn btn-ghost'
              type='button'
              onClick={() => {
                // Clean up preview URLs
                Object.values(createImagePreviews).forEach(url => URL.revokeObjectURL(url))
                setShowCreate(false)
                setCreateData('{}')
                setCreateImageFiles({})
                setCreateImagePreviews({})
                setCreateError(null)
              }}
            >
              キャンセル
            </button>
          </div>
        </form>
      )}

      {showGenerate && (
        <form className='card form-card' onSubmit={handleGenerate}>
          <h2 className='card-title'>AI レコード生成</h2>
          <div className='form-group'>
            <label className='form-label'>プロンプト</label>
            <textarea
              className='form-textarea'
              value={generatePrompt}
              onChange={(e) => setGeneratePrompt(e.target.value)}
              rows={4}
              placeholder='例: 日本の主要都市の天気情報を1件生成してください'
              required
            />
          </div>
          {generateError && <p className='error-text'>{generateError}</p>}
          <div className='form-actions'>
            <button
              className='btn btn-primary'
              type='submit'
              disabled={generating}
            >
              {generating ? '生成中...' : '生成'}
            </button>
            <button
              className='btn btn-ghost'
              type='button'
              onClick={() => setShowGenerate(false)}
            >
              キャンセル
            </button>
          </div>
        </form>
      )}

      {loading && <p className='muted'>読み込み中...</p>}
      {error && <p className='error-text'>{error}</p>}

      {!loading && records.length === 0 && (
        <p className='muted'>レコードがありません</p>
      )}

      <div className='record-list'>
        {records.map((r) => (
          <div key={r.id} className='card record-card'>
            <div className='record-meta'>
              <span className='record-id muted'>{r.id}</span>
              <span className='record-date muted'>
                {formatDate(r.created_at)}
              </span>
            </div>

            {editId === r.id ? (
              <div className='edit-inline'>
                <textarea
                  className='form-textarea'
                  value={editData}
                  onChange={(e) => setEditData(e.target.value)}
                  rows={8}
                  spellCheck={false}
                  autoFocus
                />
                {editError && <p className='error-text'>{editError}</p>}
                <div className='row-actions'>
                  <button
                    className='btn btn-primary btn-sm'
                    onClick={() => handleEditSave(r.id)}
                  >
                    保存
                  </button>
                  <button
                    className='btn btn-ghost btn-sm'
                    onClick={() => setEditId(null)}
                  >
                    キャンセル
                  </button>
                </div>
              </div>
            ) : (
              <>
                <RecordFields data={r.data} />

                {r.relations.length > 0 && (
                  <div className='canvas-panel' style={{ marginTop: '0.5rem' }}>
                    <div className='canvas-panel-header'>
                      <span className='canvas-panel-title'>
                        リレーション ({r.relations.length})
                      </span>
                    </div>
                    <div className='edge-card-list'>
                      {r.relations.map((rel) => (
                        <RelationCard key={rel.edge_id} relation={rel} />
                      ))}
                    </div>
                  </div>
                )}

                <div className='row-actions'>
                  {showMerge && (
                    <>
                      <label className='form-checkbox-label'>
                        <input
                          type='checkbox'
                          checked={mergeSourceIds.has(r.id)}
                          onChange={() => toggleMergeSelect(r.id)}
                        />
                        <span>選択</span>
                      </label>
                      {mergeSourceIds.has(r.id) && (
                        <button
                          className={`btn btn-sm ${mergeTargetId === r.id ? 'btn-primary' : 'btn-ghost'}`}
                          onClick={() => setMergeTargetId(r.id)}
                        >
                          {mergeTargetId === r.id
                            ? 'マージ先'
                            : 'マージ先に指定'}
                        </button>
                      )}
                    </>
                  )}
                  <button
                    className='btn btn-ghost btn-sm'
                    onClick={() => startEdit(r)}
                  >
                    編集
                  </button>
                  <button
                    className='btn btn-ghost btn-sm'
                    onClick={() => openCopy(r.id)}
                  >
                    コピー
                  </button>
                  <button
                    className={`btn btn-sm ${splitRecordId === r.id ? 'btn-primary' : 'btn-ghost'}`}
                    onClick={() => {
                      if (splitRecordId === r.id) {
                        setSplitRecordId(null)
                      } else {
                        setSplitRecordId(r.id)
                        setSplitData('[{}]')
                        setSplitError(null)
                      }
                    }}
                  >
                    分割
                  </button>
                  <button
                    className={`btn btn-sm ${imageRecordId === r.id ? 'btn-primary' : 'btn-ghost'}`}
                    onClick={() => {
                      if (imageRecordId === r.id) {
                        setImageRecordId(null)
                        setImagePreview(null)
                      } else {
                        setImageRecordId(r.id)
                        setImageError(null)
                        setImagePreview(null)
                      }
                    }}
                  >
                    画像
                  </button>
                  <button
                    className={`btn btn-sm ${edgeRecordId === r.id ? 'btn-primary' : 'btn-ghost'}`}
                    onClick={() => toggleEdges(r.id)}
                  >
                    エッジ
                  </button>
                  <button
                    className={`btn btn-sm ${graphRecordId === r.id ? 'btn-primary' : 'btn-ghost'}`}
                    onClick={() => toggleGraph(r.id)}
                  >
                    グラフ
                  </button>
                  <button
                    className='btn btn-danger btn-sm'
                    onClick={() => handleDelete(r.id)}
                  >
                    削除
                  </button>
                </div>
              </>
            )}

            {/* Edge panel */}
            {edgeRecordId === r.id && (
              <div className='canvas-panel'>
                <div className='canvas-panel-header'>
                  <span className='canvas-panel-title'>エッジ一覧</span>
                  <button
                    className='btn btn-ghost btn-sm'
                    onClick={() => {
                      setShowCreateEdge(!showCreateEdge)
                      setCreateEdgeError(null)
                    }}
                  >
                    {showCreateEdge ? 'キャンセル' : '+ エッジ作成'}
                  </button>
                </div>

                {showCreateEdge && (
                  <div className='edge-create-form'>
                    <div className='edge-create-row'>
                      <div
                        className='form-group'
                        style={{ flex: 1, marginBottom: 0 }}
                      >
                        <label className='form-label'>
                          To（送信先レコード ID）
                        </label>
                        <input
                          className='form-input'
                          value={edgeTo}
                          onChange={(e) => setEdgeTo(e.target.value)}
                          placeholder='例: organizations:def456'
                        />
                      </div>
                      <div
                        className='form-group'
                        style={{ flex: 0.5, marginBottom: 0 }}
                      >
                        <label className='form-label'>Label</label>
                        <input
                          className='form-input'
                          value={edgeLabel}
                          onChange={(e) => setEdgeLabel(e.target.value)}
                          placeholder='例: belongs_to'
                        />
                      </div>
                      <button
                        className='btn btn-primary btn-sm'
                        style={{ alignSelf: 'flex-end' }}
                        onClick={() => handleCreateEdge(r.id)}
                        disabled={creatingEdge}
                      >
                        {creatingEdge ? '作成中...' : '作成'}
                      </button>
                    </div>
                    {createEdgeError && (
                      <p className='error-text'>{createEdgeError}</p>
                    )}
                  </div>
                )}

                {edgesError && <p className='error-text'>{edgesError}</p>}
                {edgesLoading && <p className='muted'>読み込み中...</p>}

                {!edgesLoading && edges.length === 0 && !edgesError && (
                  <p className='muted'>エッジがありません</p>
                )}

                {edges.length > 0 && (
                  <div className='edge-card-list'>
                    {edges.map((edge) => (
                      <EdgeCard
                        key={edge.id}
                        edge={edge}
                        selfId={fullRecordId(r.id)}
                        onDelete={() => handleDeleteEdge(edge.id, r.id)}
                      />
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* Copy modal */}
            {copyRecordId === r.id && (
              <div className='canvas-panel'>
                <div className='canvas-panel-header'>
                  <span className='canvas-panel-title'>
                    レコードをコピー
                  </span>
                  <button
                    className='btn btn-ghost btn-sm'
                    onClick={() => setCopyRecordId(null)}
                  >
                    キャンセル
                  </button>
                </div>
                <div className='form-group' style={{ marginTop: '0.5rem' }}>
                  <label className='form-label'>コピー先コレクション</label>
                  <select
                    className='form-select'
                    value={copyTargetCollectionId}
                    onChange={(e) => setCopyTargetCollectionId(e.target.value)}
                  >
                    <option value=''>選択してください</option>
                    {allCollections.map((c) => (
                      <option key={c.id} value={rawId(c.id)}>
                        {c.name}
                      </option>
                    ))}
                  </select>
                </div>
                {copyError && <p className='error-text'>{copyError}</p>}
                <div className='form-actions' style={{ marginTop: '0.5rem' }}>
                  <button
                    className='btn btn-primary btn-sm'
                    disabled={copying || !copyTargetCollectionId}
                    onClick={handleCopyRecord}
                  >
                    {copying ? 'コピー中...' : 'コピー実行'}
                  </button>
                </div>
              </div>
            )}

            {/* Split panel */}
            {splitRecordId === r.id && (
              <div className='canvas-panel'>
                <div className='canvas-panel-header'>
                  <span className='canvas-panel-title'>レコードを分割</span>
                  <button
                    className='btn btn-ghost btn-sm'
                    onClick={() => setSplitRecordId(null)}
                  >
                    キャンセル
                  </button>
                </div>
                <div className='form-group' style={{ marginTop: '0.5rem' }}>
                  <label className='form-label'>
                    分割データ（JSON配列）
                  </label>
                  <textarea
                    className='form-textarea'
                    value={splitData}
                    onChange={(e) => setSplitData(e.target.value)}
                    rows={4}
                    spellCheck={false}
                    placeholder='[{}, {"name": "new1"}, {"name": "new2"}]'
                  />
                </div>
                {splitError && <p className='error-text'>{splitError}</p>}
                <div className='form-actions' style={{ marginTop: '0.5rem' }}>
                  <button
                    className='btn btn-primary btn-sm'
                    disabled={splitting}
                    onClick={() => handleSplit(r.id)}
                  >
                    {splitting ? '分割中...' : '分割実行'}
                  </button>
                </div>
              </div>
            )}

            {/* Image panel */}
            {imageRecordId === r.id && (
              <div className='canvas-panel'>
                <div className='canvas-panel-header'>
                  <span className='canvas-panel-title'>画像管理</span>
                  <button
                    className='btn btn-ghost btn-sm'
                    onClick={() => {
                      setImageRecordId(null)
                      setImagePreview(null)
                    }}
                  >
                    閉じる
                  </button>
                </div>

                {/* Existing images */}
                {r.images && r.images.length > 0 && (
                  <div style={{ marginTop: '0.5rem' }}>
                    <label className='form-label'>登録済み画像</label>
                    <div className='image-list'>
                      {r.images.map((img) => (
                        <div key={img.pointer} className='image-item'>
                          <span className='image-name'>{img.pointer}</span>
                          <div className='image-actions'>
                            <button
                              className='btn btn-ghost btn-sm'
                              onClick={() => handleViewImage(r.id, img.pointer)}
                            >
                              表示
                            </button>
                            <button
                              className='btn btn-danger btn-sm'
                              onClick={() =>
                                handleImageDelete(r.id, img.pointer)
                              }
                            >
                              削除
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Image preview */}
                {imagePreview && (
                  <div style={{ marginTop: '0.5rem' }}>
                    <img
                      src={imagePreview}
                      alt='Preview'
                      style={{
                        maxWidth: '100%',
                        maxHeight: '300px',
                        borderRadius: '4px',
                      }}
                    />
                  </div>
                )}

                {/* Upload form */}
                <div style={{ marginTop: '0.75rem' }}>
                  <label className='form-label'>新規アップロード</label>
                  <div className='form-group' style={{ marginBottom: '0.5rem' }}>
                    <input
                      className='form-input'
                      value={imageFieldName}
                      onChange={(e) => setImageFieldName(e.target.value)}
                      placeholder='フィールド名（例: thumbnail）'
                    />
                  </div>
                  <input
                    type='file'
                    accept='image/*'
                    onChange={(e) => {
                      const file = e.target.files?.[0]
                      if (file && imageFieldName.trim()) {
                        handleImageUpload(r.id, file, imageFieldName.trim())
                      } else if (file && !imageFieldName.trim()) {
                        setImageError('フィールド名を入力してください')
                      }
                    }}
                    disabled={uploading}
                  />
                </div>
                {imageError && <p className='error-text'>{imageError}</p>}
                {uploading && <p className='muted'>アップロード中...</p>}
              </div>
            )}

            {/* Graph query panel */}
            {graphRecordId === r.id && (
              <div className='canvas-panel'>
                <div className='canvas-panel-header'>
                  <span className='canvas-panel-title'>部分グラフクエリ</span>
                </div>
                <div className='query-form-row'>
                  <div
                    className='form-group'
                    style={{ flex: 0.5, marginBottom: 0 }}
                  >
                    <label className='form-label'>深度</label>
                    <input
                      className='form-input'
                      type='number'
                      min={1}
                      max={10}
                      value={queryDepth}
                      onChange={(e) => setQueryDepth(Number(e.target.value))}
                    />
                  </div>
                  <div
                    className='form-group'
                    style={{ flex: 1, marginBottom: 0 }}
                  >
                    <label className='form-label'>形式</label>
                    <select
                      className='form-select'
                      value={queryFormat}
                      onChange={(e) =>
                        setQueryFormat(
                          e.target.value as 'json-canvas' | 'markdown',
                        )
                      }
                    >
                      <option value='json-canvas'>JSON Canvas</option>
                      <option value='markdown'>Markdown</option>
                    </select>
                  </div>
                  <button
                    className='btn btn-primary btn-sm'
                    style={{ alignSelf: 'flex-end' }}
                    onClick={() => handleQuery(r.id)}
                    disabled={querying}
                  >
                    {querying ? 'クエリ中...' : 'クエリ実行'}
                  </button>
                </div>

                {queryError && <p className='error-text'>{queryError}</p>}

                {queryResult && (
                  <div className='query-result'>
                    <div className='query-result-header'>
                      <span className='query-result-meta'>
                        {queryResult.format === 'json-canvas' &&
                        queryResult.json_canvas
                          ? `${queryResult.json_canvas.nodes.length} ノード / ${queryResult.json_canvas.edges.length} エッジ`
                          : 'Markdown'}
                      </span>
                      <button
                        className='btn btn-ghost btn-sm'
                        onClick={handleCopy}
                      >
                        {copied ? 'コピー済み!' : 'コピー'}
                      </button>
                    </div>
                    {queryResult.format === 'json-canvas' &&
                    queryResult.json_canvas ? (
                      <CanvasGraph
                        nodes={queryResult.json_canvas.nodes}
                        edges={queryResult.json_canvas.edges}
                      />
                    ) : queryResult.markdown ? (
                      <RenderedMarkdown text={queryResult.markdown} />
                    ) : null}
                  </div>
                )}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}
