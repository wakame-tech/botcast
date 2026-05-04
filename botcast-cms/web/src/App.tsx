import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { Layout } from './components/Layout'
import { CollectionsPage } from './pages/CollectionsPage'
import { CollectionDetailPage } from './pages/CollectionDetailPage'
import { ScriptsPage } from './pages/ScriptsPage'
import { JobsPage } from './pages/JobsPage'

function App() {
  return (
    <BrowserRouter>
      <Layout>
        <Routes>
          <Route path='/' element={<CollectionsPage />} />
          <Route
            path='/collections/:collectionId'
            element={<CollectionDetailPage />}
          />
          <Route path='/scripts' element={<ScriptsPage />} />
          <Route path='/jobs' element={<JobsPage />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  )
}

export default App
