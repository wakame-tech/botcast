import type { ReactNode } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { useAuth } from "../contexts/AuthContext";
import "./Layout.css";

interface Props {
	children: ReactNode;
}

export function Layout({ children }: Props) {
	const { pathname } = useLocation();
	const { signOut } = useAuth();
	const navigate = useNavigate();

	function handleSignOut() {
		signOut();
		navigate("/signin");
	}

	return (
		<div className="layout">
			<nav className="nav">
				<div className="nav-brand">Botcast CMS</div>
				<ul className="nav-links">
					<li>
						<Link to="/" className={pathname === "/" ? "active" : ""}>
							Collections
						</Link>
					</li>
					<li>
						<Link
							to="/scripts"
							className={pathname === "/scripts" ? "active" : ""}
						>
							Scripts
						</Link>
					</li>
					<li>
						<Link to="/jobs" className={pathname === "/jobs" ? "active" : ""}>
							Jobs
						</Link>
					</li>
					<li>
						<button
							onClick={handleSignOut}
							style={{
								background: "none",
								border: "none",
								cursor: "pointer",
								color: "inherit",
								fontSize: "inherit",
								padding: "0",
							}}
						>
							Sign Out
						</button>
					</li>
				</ul>
			</nav>
			<main className="main">{children}</main>
		</div>
	);
}
