import { Routes, Route, Navigate } from "react-router-dom";
import { AppLayout } from "./components/layout/app-layout";
import { SearchPage } from "./pages/search-page";
import { DetailPage } from "./pages/detail-page";
import { ChatPage } from "./pages/chat-page";
import { FeedPage } from "./pages/feed-page";
import { SettingsPage } from "./pages/settings-page";

export default function App() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route path="/" element={<Navigate to="/search" replace />} />
        <Route path="/search" element={<SearchPage />} />
        <Route path="/detail/:type/:id" element={<DetailPage />} />
        <Route path="/chat" element={<ChatPage />} />
        <Route path="/feed" element={<FeedPage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Route>
    </Routes>
  );
}
