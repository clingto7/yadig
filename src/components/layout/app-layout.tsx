import { Outlet } from "react-router-dom";
import { AppSidebar } from "./app-sidebar";
import { AudioPlayer } from "@/components/audio-player";

export function AppLayout() {
  return (
    <div className="flex h-screen overflow-hidden">
      <AppSidebar />
      <main className="flex flex-1 overflow-hidden">
        <div className="flex-1 overflow-y-auto pb-14">
          <Outlet />
        </div>
        <AudioPlayer />
      </main>
    </div>
  );
}
