import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { Layout } from "./components/Layout";
import { AuthProvider, useAuth } from "./contexts/AuthContext";
import { CollectionDetailPage } from "./pages/CollectionDetailPage";
import { CollectionsPage } from "./pages/CollectionsPage";
import { JobsPage } from "./pages/JobsPage";
import { ScriptsPage } from "./pages/ScriptsPage";
import { SignInPage } from "./pages/SignInPage";

function ProtectedLayout() {
	const { isAuthenticated } = useAuth();
	if (!isAuthenticated) return <Navigate to="/signin" replace />;
	return (
		<Layout>
			<Routes>
				<Route path="/" element={<CollectionsPage />} />
				<Route
					path="/collections/:collectionId"
					element={<CollectionDetailPage />}
				/>
				<Route path="/scripts" element={<ScriptsPage />} />
				<Route path="/jobs" element={<JobsPage />} />
			</Routes>
		</Layout>
	);
}

function App() {
	return (
		<AuthProvider>
			<BrowserRouter>
				<Routes>
					<Route path="/signin" element={<SignInPage />} />
					<Route path="/*" element={<ProtectedLayout />} />
				</Routes>
			</BrowserRouter>
		</AuthProvider>
	);
}

export default App;
