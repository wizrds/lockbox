import http, { Params, RefinedResponse, ResponseType } from 'k6/http';
import { check, fail } from 'k6';

export type LockboxConfig = {
  baseUrl: string;
  apiBaseUrl: string;
  defaultNamespace: string;
  tenantId?: string;
  timeout: string;
};

export type CreateApiKeyPayload = {
  owner: string;
  scope?: string;
  tag?: string;
  expires_at?: string;
  metadata?: Record<string, string>;
};

export type IntrospectApiKeyPayload = {
  token: string;
  scope?: string;
  tags?: Array<string | null>;
};

export type CreatedNamespace = {
  name: string;
  created_at: string;
  is_default: boolean;
};

export type CreatedTag = {
  namespace: string;
  name: string;
  created_at: string;
};

export type CreatedApiKey = {
  id: string;
  namespace: string;
  key: string;
  created_at: string;
  owner: string;
  scope?: string;
  tag?: string;
  expires_at?: string;
  metadata: Record<string, string>;
};

export type ApiKeyRecord = {
  id: string;
  namespace: string;
  short_key: string;
  created_at: string;
  owner: string;
  scope?: string;
  tag?: string;
  revoked: boolean;
  revoked_at?: string;
  expires_at?: string;
  last_used_at?: string;
  metadata: Record<string, string>;
};

export type IntrospectApiKeyResponse = {
  valid: boolean;
  key: ApiKeyRecord | null;
};

export type Page<T> = {
  items: T[];
  count: number;
  next_page: number | null;
  previous_page: number | null;
};

type ErrorBody = {
  code?: number;
  message?: string;
  fields?: Array<unknown>;
};

export function getConfig(): LockboxConfig {
  const baseUrl = stripTrailingSlash(__ENV.LOCKBOX_BASE_URL || __ENV.BASE_URL || 'http://127.0.0.1:8087');

  return {
    baseUrl,
    apiBaseUrl: `${baseUrl}/v1`,
    defaultNamespace: __ENV.LOCKBOX_DEFAULT_NAMESPACE || 'apik',
    tenantId: __ENV.LOCKBOX_TENANT_ID,
    timeout: __ENV.LOCKBOX_TIMEOUT || '30s',
  };
}

export function shortName(prefix: string): string {
  const safePrefix = prefix.slice(0, 2);
  const randomPart = Math.floor(Math.random() * 1679616)
    .toString(36)
    .padStart(4, '0')
    .slice(-4);

  return `${safePrefix}${randomPart}`.slice(0, 6);
}

export function ownerName(prefix: string): string {
  // k6 only defines __VU/__ITER in the VU execution context (not in setup/teardown).
  const vu = typeof __VU !== 'undefined' ? __VU : 0;
  const iter = typeof __ITER !== 'undefined' ? __ITER : 0;
  return `${prefix}-${vu}-${iter}-${Date.now()}`;
}

export function createNamespace(config: LockboxConfig, name: string): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'POST', '/namespaces', { name }, 'create_namespace');
}

export function findNamespaces(config: LockboxConfig): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'GET', '/namespaces', undefined, 'find_namespaces');
}

export function getNamespace(config: LockboxConfig, name: string): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'GET', `/namespaces/${name}`, undefined, 'get_namespace');
}

export function deleteNamespace(config: LockboxConfig, name: string): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'DELETE', `/namespaces/${name}`, undefined, 'delete_namespace');
}

export function createTag(
  config: LockboxConfig,
  namespace: string,
  name: string,
): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'POST', `/namespaces/${namespace}/tags`, { name }, 'create_tag');
}

export function findTags(config: LockboxConfig, namespace: string): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'GET', `/namespaces/${namespace}/tags`, undefined, 'find_tags');
}

export function getTag(config: LockboxConfig, namespace: string, name: string): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'GET', `/namespaces/${namespace}/tags/${name}`, undefined, 'get_tag');
}

export function deleteTag(
  config: LockboxConfig,
  namespace: string,
  name: string,
): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'DELETE', `/namespaces/${namespace}/tags/${name}`, undefined, 'delete_tag');
}

export function createApiKey(
  config: LockboxConfig,
  payload: CreateApiKeyPayload,
): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'POST', '/api_keys', payload, 'create_api_key');
}

export function findApiKeys(config: LockboxConfig): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'GET', '/api_keys', undefined, 'find_api_keys');
}

export function getApiKey(config: LockboxConfig, id: string): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'GET', `/api_keys/${id}`, undefined, 'get_api_key');
}

export function deleteApiKey(config: LockboxConfig, id: string): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'DELETE', `/api_keys/${id}`, undefined, 'delete_api_key');
}

export function introspectApiKey(
  config: LockboxConfig,
  payload: IntrospectApiKeyPayload,
): RefinedResponse<ResponseType | undefined> {
  return jsonRequest(config, 'POST', '/api_keys/introspect', payload, 'introspect_api_key');
}

export function expectStatus(
  response: RefinedResponse<ResponseType | undefined>,
  expectedStatus: number,
  description: string,
): void {
  const ok = check(response, {
    [description]: (res) => res.status === expectedStatus,
  });

  if (!ok) {
    fail(`${description}: ${describeError(response)}`);
  }
}

export function parseJson<T>(response: RefinedResponse<ResponseType | undefined>): T {
  return response.json() as T;
}

export function assert(condition: boolean, message: string): void {
  if (!condition) {
    fail(message);
  }
}

export function safeDeleteApiKey(config: LockboxConfig, id?: string): void {
  if (!id) {
    return;
  }

  const response = deleteApiKey(config, id);
  if (response.status !== 204 && response.status !== 404) {
    fail(`cleanup api key failed: ${describeError(response)}`);
  }
}

export function safeDeleteTag(config: LockboxConfig, namespace: string, name?: string): void {
  if (!name) {
    return;
  }

  const response = deleteTag(config, namespace, name);
  if (response.status !== 204 && response.status !== 404) {
    fail(`cleanup tag failed: ${describeError(response)}`);
  }
}

export function safeDeleteNamespace(config: LockboxConfig, name?: string): void {
  if (!name) {
    return;
  }

  const response = deleteNamespace(config, name);
  if (response.status !== 204 && response.status !== 404) {
    fail(`cleanup namespace failed: ${describeError(response)}`);
  }
}

function jsonRequest(
  config: LockboxConfig,
  method: string,
  path: string,
  body: unknown,
  name: string,
): RefinedResponse<ResponseType | undefined> {
  const params: Params = {
    headers: jsonHeaders(config),
    timeout: config.timeout,
    tags: { name },
  };
  const payload = body === undefined ? null : JSON.stringify(body);

  return http.request(method, `${config.apiBaseUrl}${path}`, payload, params);
}

function jsonHeaders(config: LockboxConfig): Record<string, string> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };

  if (config.tenantId) {
    headers['X-Tenant-Id'] = config.tenantId;
  }

  return headers;
}

function describeError(response: RefinedResponse<ResponseType | undefined>): string {
  let details = `${response.request.method} ${response.url} returned ${response.status}`;

  if (!response.body) {
    return details;
  }

  try {
    const data = response.json() as ErrorBody;
    if (data.message) {
      details = `${details}: ${data.message}`;
    } else if (data.fields) {
      details = `${details}: validation failed`;
    }
  } catch (_error) {
    details = `${details}: ${String(response.body).slice(0, 240)}`;
  }

  return details;
}

function stripTrailingSlash(value: string): string {
  return value.replace(/\/+$/, '');
}
