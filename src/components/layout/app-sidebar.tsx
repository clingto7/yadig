import { Search, MessageSquare, Newspaper, Settings } from "lucide-react";
import { NavLink } from "react-router-dom";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/search", label: "Search", icon: Search },
  { to: "/chat", label: "Chat", icon: MessageSquare },
  { to: "/feed", label: "Feed", icon: Newspaper },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function AppSidebar() {
  return (
    <aside className="flex w-56 flex-col border-r border-border bg-card">
      <div className="flex h-14 items-center px-4">
        <h1 className="text-xl font-bold text-primary">yadig</h1>
      </div>
      <nav className="flex-1 space-y-1 px-2 py-2">
        {navItems.map(({ to, label, icon: Icon }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                isActive
                  ? "bg-primary/10 text-primary"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              )
            }
          >
            <Icon className="h-4 w-4" />
            {label}
          </NavLink>
        ))}
      </nav>
      <div className="border-t border-border p-3 text-xs text-muted-foreground">
        v0.1.0
      </div>
    </aside>
  );
}
