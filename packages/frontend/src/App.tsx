import { type FormEvent, useMemo, useState } from 'react'
import { ConnectError } from '@connectrpc/connect'
import { Loader2 } from 'lucide-react'
import { toast } from 'sonner'

import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { MANAGER_GRPC_BASE_URL, nodeManagerClient } from '@/lib/rpc'
import { PortMappingMode, type GetNodeConfigResponse } from '@/gen/proto/manager_pb'

const MODE_LABELS: Record<PortMappingMode, string> = {
  [PortMappingMode.UNSPECIFIED]: 'Unspecified',
  [PortMappingMode.SERVER]: 'Server',
  [PortMappingMode.CLIENT]: 'Client',
}

const getModeLabel = (mode: PortMappingMode) => MODE_LABELS[mode] ?? 'Unknown'

const App = () => {
  const [nodeName, setNodeName] = useState('')
  const [loading, setLoading] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const [response, setResponse] = useState<GetNodeConfigResponse | null>(null)

  const portMapping = response?.portMapping

  const portMappingDetails = useMemo(() => {
    if (!portMapping) {
      return { hasConfig: false, isJson: true, pretty: '' }
    }

    if (!portMapping.configJson) {
      return { hasConfig: true, isJson: true, pretty: '（空配置）' }
    }

    try {
      const parsed = JSON.parse(portMapping.configJson)
      return {
        hasConfig: true,
        isJson: true,
        pretty: JSON.stringify(parsed, null, 2),
      }
    } catch {
      return { hasConfig: true, isJson: false, pretty: portMapping.configJson }
    }
  }, [portMapping])

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const trimmed = nodeName.trim()
    if (!trimmed) {
      const message = '请输入节点名称'
      setErrorMessage(message)
      setResponse(null)
      toast.error(message)
      return
    }

    setLoading(true)
    setErrorMessage(null)

    try {
      const result = await nodeManagerClient.getNodeConfig({ name: trimmed })
      setResponse(result)
      toast.success(`已加载节点「${result.name || trimmed}」的配置`)
      if (!result.portMapping) {
        toast.info('该节点没有配置端口映射')
      }
    } catch (err) {
      let message = '无法获取节点配置'
      if (err instanceof ConnectError) {
        message = err.rawMessage
      } else if (err instanceof Error) {
        message = err.message
      }
      setErrorMessage(message)
      setResponse(null)
      toast.error(message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <main className="bg-background text-foreground">
      <div className="mx-auto flex min-h-[calc(100vh-3.5rem)] w-full max-w-3xl flex-col gap-8 px-4 py-12">
        <section className="space-y-3 text-center">
          <h1 className="text-3xl font-bold tracking-tight sm:text-4xl">Laval 节点配置查询</h1>
          <p className="text-base text-muted-foreground">
            通过浏览器内的 gRPC-Web 请求调用 <code>GetNodeConfig</code>，实时查看节点的端口映射配置。
          </p>
          <p className="text-xs text-muted-foreground">
            当前 gRPC 服务地址：
            <code className="ml-2 rounded bg-muted px-2 py-1 font-mono text-xs">
              {MANAGER_GRPC_BASE_URL}
            </code>
          </p>
        </section>

        <Card>
          <CardHeader>
            <CardTitle>查询节点配置</CardTitle>
            <CardDescription>
              输入节点名称，调用 <code>GetNodeConfig</code> 获取节点在 Laval Manager 中登记的配置。
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-6">
              <div className="space-y-2">
                <Label htmlFor="nodeName">节点名称</Label>
                <Input
                  id="nodeName"
                  placeholder="例如：edge-01"
                  autoComplete="off"
                  value={nodeName}
                  onChange={(event) => setNodeName(event.target.value)}
                />
                <p className="text-xs text-muted-foreground">提交前会自动去除首尾空格。</p>
              </div>

              {errorMessage ? (
                <Alert variant="destructive">
                  <AlertTitle>查询失败</AlertTitle>
                  <AlertDescription>{errorMessage}</AlertDescription>
                </Alert>
              ) : null}

              <div className="flex flex-wrap items-center gap-3">
                <Button type="submit" disabled={loading}>
                  {loading ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      查询中...
                    </>
                  ) : (
                    '查询配置'
                  )}
                </Button>
                {response ? (
                  <span className="text-sm text-muted-foreground">
                    已为节点
                    <span className="mx-1 font-medium text-foreground">{response.name || '（未命名）'}</span>
                    加载配置。
                  </span>
                ) : null}
              </div>
            </form>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>查询结果</CardTitle>
            <CardDescription>展示 gRPC 返回的数据结构。</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {response ? (
              <div className="space-y-4">
                <div>
                  <h3 className="text-sm font-medium text-muted-foreground">节点名称</h3>
                  <p className="text-lg font-semibold text-foreground">{response.name}</p>
                </div>
                <Separator />
                <div className="space-y-2">
                  <h3 className="text-sm font-medium text-muted-foreground">端口映射</h3>
                  {portMapping ? (
                    <div className="space-y-3 rounded-lg border bg-card p-4">
                      <div className="flex flex-wrap items-center gap-2 text-sm">
                        <span className="font-medium">模式</span>
                        <span className="rounded bg-secondary px-2 py-1 text-secondary-foreground">
                          {getModeLabel(portMapping.mode)}
                        </span>
                      </div>
                      {portMappingDetails.hasConfig ? (
                        <div className="space-y-2 text-sm">
                          <span className="font-medium">配置内容</span>
                          <pre className="max-h-64 overflow-auto rounded-md bg-muted p-3 text-sm leading-relaxed">
                            <code>{portMappingDetails.pretty}</code>
                          </pre>
                          {!portMappingDetails.isJson ? (
                            <p className="text-xs text-muted-foreground">
                              配置不是有效的 JSON，已按原始文本展示。
                            </p>
                          ) : null}
                        </div>
                      ) : (
                        <p className="text-sm text-muted-foreground">该节点未提供端口映射的详细配置。</p>
                      )}
                    </div>
                  ) : (
                    <p className="text-sm text-muted-foreground">该节点没有配置端口映射。</p>
                  )}
                </div>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                查询后将展示节点名称以及可能存在的端口映射信息。
              </p>
            )}
          </CardContent>
        </Card>
      </div>
    </main>
  )
}

export default App
