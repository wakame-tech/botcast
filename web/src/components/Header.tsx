import { Link } from "@tanstack/react-router"
import { useSession } from "../hooks/useSession"

export function Header() {
    const { session, signOut } = useSession()
    const buttonStyle = `hover:bg-orange-500 border border-1 p-2 font-bold rounded-lg`

    return (
        <>
            <div className="p-2 flex items-baseline gap-2">
                <Link to="/" className="font-bold text-teal-700 text-3xl no-underline">
                    Botcast
                </Link>{' '}
                <Link to="/episodes" className="[&.active]:font-bold text-xl no-underline">
                    エピソード
                </Link>
                <Link to="/tasks" className="[&.active]:font-bold text-xl no-underline">
                    タスク
                </Link>

                <div className="flex-grow" />
                <>
                    {session && (
                        <>
                            <Link to="/users/$userId" params={{ userId: session.user.id }} className="no-underline">
                                プロフィール
                            </Link>
                            <button onClick={() => signOut()} className={buttonStyle}>
                                サインアウト
                            </button>
                        </>
                    )}
                    {!session &&
                        <Link to="/signin">
                            <button className={buttonStyle}>
                                サインイン
                            </button>
                        </Link>
                    }
                </>
            </div >
            <hr />
        </>
    )
}
