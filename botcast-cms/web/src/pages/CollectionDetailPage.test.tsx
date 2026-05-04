import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { CollectionDetailPage } from './CollectionDetailPage'
import * as client from '../api/client'
import type { Collection, RecordWithRelations } from '../types'

// Mock the API client
vi.mock('../api/client', () => ({
  collectionsApi: {
    read: vi.fn(),
    list: vi.fn(),
  },
  recordsApi: {
    listWithRelations: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    delete: vi.fn(),
    generate: vi.fn(),
    copy: vi.fn(),
    merge: vi.fn(),
    split: vi.fn(),
  },
  imagesApi: {
    upload: vi.fn(),
    get: vi.fn(),
    delete: vi.fn(),
  },
  canvasApi: {
    listEdges: vi.fn(),
    createEdge: vi.fn(),
    deleteEdge: vi.fn(),
    query: vi.fn(),
  },
  rawId: (id: string) => id.replace(/^[^:]+:/, ''),
}))

const mockCollection: Collection = {
  id: 'collection:test123',
  name: 'Test Collection',
  schema: {
    type: 'object',
    properties: {
      name: { type: 'string' },
      count: { type: 'number' },
    },
  },
  created_at: '2024-01-01T00:00:00Z',
}

const mockRecords: RecordWithRelations[] = [
  {
    id: 'test123:rec1',
    data: { name: 'Record 1', count: 10 },
    created_at: '2024-01-01T00:00:00Z',
    updated_at: null,
    relations: [],
  },
  {
    id: 'test123:rec2',
    data: { name: 'Record 2', count: 20 },
    created_at: '2024-01-02T00:00:00Z',
    updated_at: null,
    relations: [],
  },
]

const renderWithRouter = (collectionId: string = 'test123') => {
  return render(
    <MemoryRouter initialEntries={[`/collections/${collectionId}`]}>
      <Routes>
        <Route
          path="/collections/:collectionId"
          element={<CollectionDetailPage />}
        />
      </Routes>
    </MemoryRouter>,
  )
}

describe('CollectionDetailPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(client.collectionsApi.read).mockResolvedValue(mockCollection)
    vi.mocked(client.recordsApi.listWithRelations).mockResolvedValue(
      mockRecords,
    )
    vi.mocked(client.collectionsApi.list).mockResolvedValue([
      mockCollection,
      {
        id: 'collection:other456',
        name: 'Other Collection',
        schema: {},
        created_at: '2024-01-01T00:00:00Z',
      },
    ])
  })

  describe('Copy record', () => {
    it('should show copy modal when clicking copy button', async () => {
      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const copyButtons = screen.getAllByText('コピー')
      await userEvent.click(copyButtons[0])

      await waitFor(() => {
        expect(screen.getByText('レコードをコピー')).toBeInTheDocument()
        expect(screen.getByText('コピー先コレクション')).toBeInTheDocument()
      })
    })

    it('should copy record to another collection', async () => {
      vi.mocked(client.recordsApi.copy).mockResolvedValue({
        id: 'other456:newrec',
        data: { name: 'Record 1', count: 10 },
        created_at: '2024-01-01T00:00:00Z',
        updated_at: null,
      })

      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const copyButtons = screen.getAllByText('コピー')
      await userEvent.click(copyButtons[0])

      await waitFor(() => {
        expect(screen.getByText('コピー先コレクション')).toBeInTheDocument()
      })

      const select = screen.getByRole('combobox')
      await userEvent.selectOptions(select, 'other456')

      const executeButton = screen.getByText('コピー実行')
      await userEvent.click(executeButton)

      await waitFor(() => {
        expect(client.recordsApi.copy).toHaveBeenCalledWith(
          'test123',
          'test123:rec1',
          { target_collection_id: 'other456' },
        )
      })
    })

    it('should disable copy button when no collection selected', async () => {
      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const copyButtons = screen.getAllByText('コピー')
      await userEvent.click(copyButtons[0])

      await waitFor(() => {
        const executeButton = screen.getByText('コピー実行')
        expect(executeButton).toBeDisabled()
      })
    })
  })

  describe('Merge records', () => {
    it('should show merge mode UI when clicking merge button', async () => {
      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const mergeButton = screen.getByText('マージ')
      await userEvent.click(mergeButton)

      await waitFor(() => {
        expect(screen.getByText('レコードマージ')).toBeInTheDocument()
        expect(screen.getByText('マージ解除')).toBeInTheDocument()
      })
    })

    it('should allow selecting records for merge', async () => {
      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const mergeButton = screen.getByText('マージ')
      await userEvent.click(mergeButton)

      await waitFor(() => {
        const checkboxes = screen.getAllByRole('checkbox')
        expect(checkboxes.length).toBeGreaterThanOrEqual(2)
      })
    })

    it('should merge records successfully', async () => {
      vi.mocked(client.recordsApi.merge).mockResolvedValue({
        id: 'test123:rec1',
        data: { name: 'Record 1', count: 30 },
        created_at: '2024-01-01T00:00:00Z',
        updated_at: null,
      })

      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const mergeButton = screen.getByText('マージ')
      await userEvent.click(mergeButton)

      await waitFor(() => {
        const checkboxes = screen.getAllByLabelText('選択')
        expect(checkboxes.length).toBeGreaterThanOrEqual(2)
      })

      const checkboxes = screen.getAllByLabelText('選択')
      await userEvent.click(checkboxes[0])
      await userEvent.click(checkboxes[1])

      await waitFor(() => {
        const targetButtons = screen.getAllByText('マージ先に指定')
        expect(targetButtons.length).toBeGreaterThan(0)
      })

      const targetButtons = screen.getAllByText('マージ先に指定')
      await userEvent.click(targetButtons[0])

      const executeButton = screen.getByText(/マージ実行/)
      await userEvent.click(executeButton)

      await waitFor(() => {
        expect(client.recordsApi.merge).toHaveBeenCalled()
      })
    })

    it('should disable merge execution with less than 2 records', async () => {
      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const mergeButton = screen.getByText('マージ')
      await userEvent.click(mergeButton)

      await waitFor(() => {
        const executeButton = screen.getByText(/マージ実行/)
        expect(executeButton).toBeDisabled()
      })
    })
  })

  describe('Split record', () => {
    it('should show split panel when clicking split button', async () => {
      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const splitButtons = screen.getAllByText('分割')
      await userEvent.click(splitButtons[0])

      await waitFor(() => {
        expect(screen.getByText('レコードを分割')).toBeInTheDocument()
        expect(screen.getByText('分割データ（JSON配列）')).toBeInTheDocument()
      })
    })

    it('should split record successfully', async () => {
      vi.mocked(client.recordsApi.split).mockResolvedValue({
        original: mockRecords[0],
        new_records: [
          {
            id: 'test123:rec3',
            data: { name: 'Split 1' },
            created_at: '2024-01-01T00:00:00Z',
            updated_at: null,
          },
        ],
      })

      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const splitButtons = screen.getAllByText('分割')
      await userEvent.click(splitButtons[0])

      await waitFor(() => {
        const textarea = screen.getByPlaceholderText(
          '[{}, {"name": "new1"}, {"name": "new2"}]',
        )
        expect(textarea).toBeInTheDocument()
      })

      const textarea = screen.getByPlaceholderText(
        '[{}, {"name": "new1"}, {"name": "new2"}]',
      )
      // Use fireEvent instead of userEvent for JSON with special characters
      fireEvent.change(textarea, { target: { value: '[{"name": "Split 1"}]' } })

      const executeButton = screen.getByText('分割実行')
      await userEvent.click(executeButton)

      await waitFor(() => {
        expect(client.recordsApi.split).toHaveBeenCalled()
      })
    })

    it('should show error for invalid JSON', async () => {
      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const splitButtons = screen.getAllByText('分割')
      await userEvent.click(splitButtons[0])

      const textarea = screen.getByPlaceholderText(
        '[{}, {"name": "new1"}, {"name": "new2"}]',
      )
      await userEvent.clear(textarea)
      await userEvent.type(textarea, 'invalid json')

      const executeButton = screen.getByText('分割実行')
      await userEvent.click(executeButton)

      await waitFor(() => {
        expect(
          screen.getByText('Splits は有効な JSON 配列で入力してください'),
        ).toBeInTheDocument()
      })
    })
  })

  describe('Image management', () => {
    it('should show image panel when clicking image button', async () => {
      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const imageButtons = screen.getAllByText('画像')
      await userEvent.click(imageButtons[0])

      await waitFor(() => {
        expect(screen.getByText('画像管理')).toBeInTheDocument()
        expect(screen.getByText('新規アップロード')).toBeInTheDocument()
      })
    })

    it('should show error when field name is empty on upload', async () => {
      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const imageButtons = screen.getAllByText('画像')
      await userEvent.click(imageButtons[0])

      await waitFor(() => {
        expect(screen.getByText('画像管理')).toBeInTheDocument()
      })

      const file = new File(['test'], 'test.png', { type: 'image/png' })
      const fileInput = screen.getByAcceptingFiles('image/*')
      await userEvent.upload(fileInput, file)

      await waitFor(() => {
        expect(
          screen.getByText('フィールド名を入力してください'),
        ).toBeInTheDocument()
      })
    })

    it('should display existing images when record has images', async () => {
      const recordsWithImages: RecordWithRelations[] = [
        {
          ...mockRecords[0],
          images: [
            {
              pointer: 'thumbnail',
              content_type: 'image/png',
              size: 1024,
              url: '/images/test.png',
            },
          ],
        },
        mockRecords[1],
      ]
      vi.mocked(client.recordsApi.listWithRelations).mockResolvedValue(
        recordsWithImages,
      )

      renderWithRouter()
      await waitFor(() => {
        expect(screen.getByText('Record 1')).toBeInTheDocument()
      })

      const imageButtons = screen.getAllByText('画像')
      await userEvent.click(imageButtons[0])

      await waitFor(() => {
        expect(screen.getByText('登録済み画像')).toBeInTheDocument()
        expect(screen.getByText('thumbnail')).toBeInTheDocument()
      })
    })
  })
})

// Helper to get file input by accept attribute
const getFileInputByAccept = (accept: string): HTMLInputElement => {
  const inputs = document.querySelectorAll<HTMLInputElement>(
    `input[type="file"][accept="${accept}"]`,
  )
  if (inputs.length === 0) {
    throw new Error(`No file input found with accept="${accept}"`)
  }
  return inputs[0]
}

// Extend screen with custom query
declare module '@testing-library/react' {
  interface Screen {
    getByAcceptingFiles(accept: string): HTMLInputElement
  }
}

screen.getByAcceptingFiles = getFileInputByAccept
