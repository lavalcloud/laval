import { createRootRoute, Link, Outlet } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools'

import { Toaster } from '@/components/ui/sonner'

const RootLayout = () => (
  <div className="min-h-screen bg-background text-foreground">
    <header className="border-b bg-card/40 backdrop-blur">
      <div className="mx-auto flex h-14 w-full max-w-5xl items-center gap-6 px-4">
        <span className="text-base font-semibold tracking-tight">Laval Manager</span>
        <nav className="flex items-center gap-4 text-sm font-medium text-muted-foreground">
          <Link to="/" className="transition-colors hover:text-foreground [&.active]:text-foreground">
            节点查询
          </Link>
          <Link to="/about" className="transition-colors hover:text-foreground [&.active]:text-foreground">
            关于
          </Link>
        </nav>
      </div>
    </header>
    <Outlet />
    <Toaster richColors />
    <TanStackRouterDevtools position="bottom-right" />
  </div>
)

export const Route = createRootRoute({ component: RootLayout })