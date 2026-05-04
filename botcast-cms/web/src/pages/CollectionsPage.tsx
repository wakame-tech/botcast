// @specre 01KNM2BBT598J55K9BMM975FCZ
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { collectionsApi, rawId } from '../api/client'
import type { Collection } from '../types'
import './Page.css'

const DEFAULT_SCHEMA = JSON.stringify(
  {
    type: 'object',
    properties: {
      title: { type: 'string', description: 'Title' },
    },
    required: ['title'],
    additionalProperties: false,
  },
  null,
  2,
)

export function CollectionsPage() {
  const [collections, setCollections] = useState<Collection[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const [showCreate, setShowCreate] = useState(false)
  const [createName, setCreateName] = useState('')
  const [createSchema, setCreateSchema] = useState(DEFAULT_SCHEMA)
  const [createError, setCreateError] = useState<string | null>(null)
  const [creating, setCreating] = useState(false)

  const [editId, setEditId] = useState<string | null>(null)
  const [editName, setEditName] = useState('')
  const [editError, setEditError] = useState<string | null>(null)

  const load = async () => {
    try {
      setLoading(true)
      const data = await collectionsApi.list()
      setCollections(data)
      setError(null)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    load()
  }, [])

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    setCreateError(null)
    try {
      JSON.parse(createSchema)
    } catch {
      setCreateError('Schema は有効な JSON で入力してください')
      return
    }
    try {
      setCreating(true)
      await collectionsApi.create({
        name: createName,
        schema: JSON.parse(createSchema),
      })
      setCreateName('')
      setCreateSchema(DEFAULT_SCHEMA)
      setShowCreate(false)
      await load()
    } catch (e) {
      setCreateError(String(e))
    } finally {
      setCreating(false)
    }
  }

  const handleDelete = async (id: string) => {
    if (!confirm('このコレクションを削除しますか？')) return
    try {
      await collectionsApi.delete(id)
      await load()
    } catch (e) {
      alert(String(e))
    }
  }

  const handleEditSave = async (id: string) => {
    setEditError(null)
    try {
      await collectionsApi.update(id, editName)
      setEditId(null)
      await load()
    } catch (e) {
      setEditError(String(e))
    }
  }

  const startEdit = (c: Collection) => {
    setEditId(c.id)
    setEditName(c.name)
    setEditError(null)
  }

  return (
    <div className='page'>
      <div className='page-header'>
        <h1 className='page-title'>Collections</h1>
        <button
          className='btn btn-primary'
          onClick={() => setShowCreate(!showCreate)}
        >
          {showCreate ? 'キャンセル' : '+ 新規作成'}
        </button>
      </div>

      {showCreate && (
        <form className='card form-card' onSubmit={handleCreate}>
          <h2 className='card-title'>新規コレクション</h2>
          <div className='form-group'>
            <label className='form-label'>名前</label>
            <input
              className='form-input'
              type='text'
              value={createName}
              onChange={(e) => setCreateName(e.target.value)}
              placeholder='例: articles'
              required
            />
          </div>
          <div className='form-group'>
            <label className='form-label'>JSON Schema</label>
            <textarea
              className='form-textarea'
              value={createSchema}
              onChange={(e) => setCreateSchema(e.target.value)}
              rows={12}
              spellCheck={false}
            />
          </div>
          {createError && <p className='error-text'>{createError}</p>}
          <div className='form-actions'>
            <button
              className='btn btn-primary'
              type='submit'
              disabled={creating}
            >
              {creating ? '作成中...' : '作成'}
            </button>
          </div>
        </form>
      )}

      {loading && <p className='muted'>読み込み中...</p>}
      {error && <p className='error-text'>{error}</p>}

      {!loading && collections.length === 0 && (
        <p className='muted'>コレクションがありません</p>
      )}

      <div className='collection-list'>
        {collections.map((c) => (
          <div key={c.id} className='card collection-card'>
            {editId === c.id ? (
              <div className='edit-inline'>
                <input
                  className='form-input'
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  autoFocus
                />
                {editError && <p className='error-text'>{editError}</p>}
                <div className='row-actions'>
                  <button
                    className='btn btn-primary btn-sm'
                    onClick={() => handleEditSave(c.id)}
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
                <div className='collection-info'>
                  <Link
                    to={`/collections/${rawId(c.id)}`}
                    className='collection-name'
                  >
                    {c.name}
                  </Link>
                  <span className='collection-id muted'>{c.id}</span>
                </div>
                <div className='row-actions'>
                  <Link
                    to={`/collections/${rawId(c.id)}`}
                    className='btn btn-ghost btn-sm'
                  >
                    詳細
                  </Link>
                  <button
                    className='btn btn-ghost btn-sm'
                    onClick={() => startEdit(c)}
                  >
                    名前変更
                  </button>
                  <button
                    className='btn btn-danger btn-sm'
                    onClick={() => handleDelete(c.id)}
                  >
                    削除
                  </button>
                </div>
              </>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}
