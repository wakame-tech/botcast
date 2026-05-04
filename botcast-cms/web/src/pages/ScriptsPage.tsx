// @specre 01KNM2BBT5XQ00DP2VDMHZ4P17
import { useState } from 'react'
import { Link } from 'react-router-dom'
import { scriptsApi } from '../api/client'
import type { ExecuteScriptRequest, ExecuteScriptResponse } from '../types'
import './Page.css'

export function ScriptsPage() {
  const [language, setLanguage] = useState<'python3' | 'nodejs'>('python3')
  const [code, setCode] = useState('')
  const [preload, setPreload] = useState('')
  const [enableNetwork, setEnableNetwork] = useState(false)
  const [executing, setExecuting] = useState(false)
  const [result, setResult] = useState<ExecuteScriptResponse | null>(null)
  const [error, setError] = useState<string | null>(null)

  const handleExecute = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    setResult(null)

    const request: ExecuteScriptRequest = {
      language,
      code,
      ...(preload && { preload }),
      enable_network: enableNetwork,
    }

    try {
      setExecuting(true)
      const response = await scriptsApi.execute(request)
      setResult(response)
    } catch (e) {
      setError(String(e))
    } finally {
      setExecuting(false)
    }
  }

  return (
    <div className='page'>
      <div className='breadcrumb'>
        <Link to='/'>Collections</Link>
        <span className='muted'> / </span>
        <span>Scripts</span>
      </div>

      <div className='page-header'>
        <h1 className='page-title'>スクリプト実行</h1>
      </div>

      <form className='card form-card' onSubmit={handleExecute}>
        <div className='form-group'>
          <label className='form-label'>言語</label>
          <select
            className='form-select'
            value={language}
            onChange={(e) =>
              setLanguage(e.target.value as 'python3' | 'nodejs')
            }
          >
            <option value='python3'>Python 3</option>
            <option value='nodejs'>Node.js</option>
          </select>
        </div>

        <div className='form-group'>
          <label className='form-label'>コード（結果は stdout に出力）</label>
          <textarea
            className='form-textarea'
            value={code}
            onChange={(e) => setCode(e.target.value)}
            rows={12}
            spellCheck={false}
            placeholder={
              language === 'python3'
                ? 'print("Hello, World!")'
                : 'console.log("Hello, World!")'
            }
            required
          />
        </div>

        <div className='form-group'>
          <label className='form-label'>Preload（オプション）</label>
          <textarea
            className='form-textarea'
            value={preload}
            onChange={(e) => setPreload(e.target.value)}
            rows={4}
            spellCheck={false}
            placeholder='実行前に読み込むコード（オプション）'
          />
        </div>

        <div className='form-group'>
          <label className='form-checkbox-label'>
            <input
              type='checkbox'
              checked={enableNetwork}
              onChange={(e) => setEnableNetwork(e.target.checked)}
            />
            <span>ネットワークアクセスを有効化</span>
          </label>
        </div>

        <div className='form-actions'>
          <button
            className='btn btn-primary'
            type='submit'
            disabled={executing}
          >
            {executing ? '実行中...' : '実行'}
          </button>
        </div>
      </form>

      {error && (
        <div className='card'>
          <h2 className='card-title error-text'>エラー</h2>
          <pre className='code-block error-text'>{error}</pre>
        </div>
      )}

      {result && (
        <div className='card'>
          <h2 className='card-title'>実行結果</h2>
          <div className='form-group'>
            <label className='form-label'>
              ステータスコード: {result.code}
            </label>
            <label className='form-label'>メッセージ: {result.message}</label>
          </div>

          {result.data.stdout && (
            <div className='form-group'>
              <label className='form-label'>標準出力 (stdout)</label>
              <pre className='code-block'>{result.data.stdout}</pre>
            </div>
          )}

          {result.data.error && (
            <div className='form-group'>
              <label className='form-label error-text'>エラー出力</label>
              <pre className='code-block error-text'>{result.data.error}</pre>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
