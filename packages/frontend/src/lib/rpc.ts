import { createClient } from '@connectrpc/connect'
import { createGrpcWebTransport } from '@connectrpc/connect-web'

import { NodeManager } from '@/gen/proto/manager_pb'

const DEFAULT_BASE_URL = 'http://localhost:50051'

export const MANAGER_GRPC_BASE_URL =
  import.meta.env.VITE_MANAGER_GRPC_URL ?? DEFAULT_BASE_URL

const transport = createGrpcWebTransport({
  baseUrl: MANAGER_GRPC_BASE_URL,
})

export const nodeManagerClient = createClient(NodeManager, transport)
