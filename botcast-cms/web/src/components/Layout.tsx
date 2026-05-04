import { Link, useLocation } from 'react-router-dom'
import type { ReactNode } from 'react'
import './Layout.css'

interface Props {
  children: ReactNode
}

export function Layout({ children }: Props) {
  const { pathname } = useLocation()

  return (
    <div className='layout'>
      <nav className='nav'>
        <div className='nav-brand'>Botcast CMS</div>
        <ul className='nav-links'>
          <li>
            <Link to='/' className={pathname === '/' ? 'active' : ''}>
              Collections
            </Link>
          </li>
          <li>
            <Link
              to='/scripts'
              className={pathname === '/scripts' ? 'active' : ''}
            >
              Scripts
            </Link>
          </li>
          <li>
            <Link to='/jobs' className={pathname === '/jobs' ? 'active' : ''}>
              Jobs
            </Link>
          </li>
        </ul>
      </nav>
      <main className='main'>{children}</main>
    </div>
  )
}
