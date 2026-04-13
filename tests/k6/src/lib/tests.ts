import { check, sleep } from 'k6';

import {
  ApiKeyRecord,
  CreatedApiKey,
  CreatedNamespace,
  CreatedTag,
  IntrospectApiKeyResponse,
  LockboxConfig,
  Page,
  assert,
  createApiKey,
  createNamespace,
  createTag,
  deleteApiKey,
  expectStatus,
  findApiKeys,
  findNamespaces,
  findTags,
  getApiKey,
  getNamespace,
  getTag,
  introspectApiKey,
  ownerName,
  parseJson,
  safeDeleteApiKey,
  safeDeleteNamespace,
  safeDeleteTag,
  shortName,
} from '@/lib/utils.js';

export type VerifyFixture = {
  tagName: string;
  apiKeyId: string;
  token: string;
  owner: string;
};

export type LifecycleFixture = {
  tagName: string;
};

export function setupVerifyFixture(config: LockboxConfig): VerifyFixture {
  const tagName = shortName('tg');
  const owner = ownerName('verify');

  const createTagResponse = createTag(config, config.defaultNamespace, tagName);
  expectStatus(createTagResponse, 201, 'creates verify fixture tag');

  const createKeyResponse = createApiKey(config, {
    owner,
    scope: 'keys:read keys:write',
    tag: tagName,
    metadata: { suite: 'k6', scenario: 'verify' },
  });
  expectStatus(createKeyResponse, 201, 'creates verify fixture api key');

  const key = parseJson<CreatedApiKey>(createKeyResponse);

  return {
    tagName,
    apiKeyId: key.id,
    token: key.key,
    owner,
  };
}

export function teardownVerifyFixture(config: LockboxConfig, fixture: VerifyFixture): void {
  safeDeleteApiKey(config, fixture.apiKeyId);
  safeDeleteTag(config, config.defaultNamespace, fixture.tagName);
}

export function setupLifecycleFixture(config: LockboxConfig): LifecycleFixture {
  const tagName = shortName('tg');
  const response = createTag(config, config.defaultNamespace, tagName);

  expectStatus(response, 201, 'creates lifecycle fixture tag');

  return { tagName };
}

export function teardownLifecycleFixture(config: LockboxConfig, fixture: LifecycleFixture): void {
  safeDeleteTag(config, config.defaultNamespace, fixture.tagName);
}

export function runVerifyIteration(config: LockboxConfig, fixture: VerifyFixture, sleepSeconds = 0): void {
  const validResponse = introspectApiKey(config, {
    token: fixture.token,
    scope: 'keys:read',
    tags: [fixture.tagName],
  });
  expectStatus(validResponse, 200, 'introspects a valid api key');

  const validBody = parseJson<IntrospectApiKeyResponse>(validResponse);
  const validCheck = check(validBody, {
    'valid introspection returns a key': (body) => body.valid === true && body.key !== null,
    'valid introspection keeps the expected owner': (body) => body.key?.owner === fixture.owner,
  });
  assert(validCheck, 'valid introspection body did not match expectations');

  const invalidScopeResponse = introspectApiKey(config, {
    token: fixture.token,
    scope: 'keys:delete',
    tags: [fixture.tagName],
  });
  expectStatus(invalidScopeResponse, 200, 'rejects an introspection with insufficient scope');

  const invalidScopeBody = parseJson<IntrospectApiKeyResponse>(invalidScopeResponse);
  assert(invalidScopeBody.valid === false && invalidScopeBody.key === null, 'insufficient scope should return an invalid result');

  if (sleepSeconds > 0) {
    sleep(sleepSeconds);
  }
}

export function runCreateVerifyDeleteIteration(
  config: LockboxConfig,
  fixture: LifecycleFixture,
  sleepSeconds = 0,
): void {
  const createResponse = createApiKey(config, {
    owner: ownerName('lifecycle'),
    scope: 'keys:read',
    tag: fixture.tagName,
    metadata: { suite: 'k6', scenario: 'lifecycle' },
  });
  expectStatus(createResponse, 201, 'creates an api key');

  const apiKey = parseJson<CreatedApiKey>(createResponse);

  const introspectResponse = introspectApiKey(config, {
    token: apiKey.key,
    scope: 'keys:read',
    tags: [fixture.tagName],
  });
  expectStatus(introspectResponse, 200, 'introspects a freshly created api key');

  const introspectBody = parseJson<IntrospectApiKeyResponse>(introspectResponse);
  assert(introspectBody.valid === true, 'freshly created key should introspect as valid');

  const deleteResponse = deleteApiKey(config, apiKey.id);
  expectStatus(deleteResponse, 204, 'deletes the api key');

  const postDeleteResponse = introspectApiKey(config, {
    token: apiKey.key,
    scope: 'keys:read',
    tags: [fixture.tagName],
  });
  expectStatus(postDeleteResponse, 200, 'rejects a deleted api key');

  const postDeleteBody = parseJson<IntrospectApiKeyResponse>(postDeleteResponse);
  assert(postDeleteBody.valid === false, 'deleted key should introspect as invalid');

  if (sleepSeconds > 0) {
    sleep(sleepSeconds);
  }
}

export function runSmokeFlow(config: LockboxConfig): void {
  const namespaceName = shortName('ns');
  const customTagName = shortName('tg');
  const defaultTagName = shortName('tg');

  try {
    const createNamespaceResponse = createNamespace(config, namespaceName);
    expectStatus(createNamespaceResponse, 201, 'creates a namespace');

    const namespace = parseJson<CreatedNamespace>(createNamespaceResponse);
    assert(namespace.name === namespaceName, 'created namespace name did not match');

    const getNamespaceResponse = getNamespace(config, namespaceName);
    expectStatus(getNamespaceResponse, 200, 'gets the namespace');

    const getNamespaceBody = parseJson<CreatedNamespace>(getNamespaceResponse);
    assert(getNamespaceBody.name === namespaceName, 'fetched namespace name did not match');

    const findNamespacesResponse = findNamespaces(config);
    expectStatus(findNamespacesResponse, 200, 'lists namespaces');

    const namespacePage = parseJson<Page<CreatedNamespace>>(findNamespacesResponse);
    assert(namespacePage.items.some((item) => item.name === namespaceName), 'created namespace should appear in the namespace list');

    const createCustomTagResponse = createTag(config, namespaceName, customTagName);
    expectStatus(createCustomTagResponse, 201, 'creates a tag in the custom namespace');

    const createDefaultTagResponse = createTag(config, config.defaultNamespace, defaultTagName);
    expectStatus(createDefaultTagResponse, 201, 'creates a tag in the default namespace');

    const getTagResponse = getTag(config, namespaceName, customTagName);
    expectStatus(getTagResponse, 200, 'gets the custom namespace tag');

    const getTagBody = parseJson<CreatedTag>(getTagResponse);
    assert(getTagBody.name === customTagName, 'fetched tag name did not match');

    const findTagsResponse = findTags(config, namespaceName);
    expectStatus(findTagsResponse, 200, 'lists tags for the custom namespace');

    const tagPage = parseJson<Page<CreatedTag>>(findTagsResponse);
    assert(tagPage.items.some((item) => item.name === customTagName), 'created tag should appear in the tag list');

    const createApiKeyResponse = createApiKey(config, {
      owner: ownerName('smoke'),
      scope: 'keys:read keys:write',
      tag: defaultTagName,
      metadata: { suite: 'k6', scenario: 'smoke' },
    });
    expectStatus(createApiKeyResponse, 201, 'creates an api key in the default namespace');

    const apiKey = parseJson<CreatedApiKey>(createApiKeyResponse);
    assert(apiKey.namespace === config.defaultNamespace, 'created key should belong to the default namespace');
    assert(apiKey.tag === defaultTagName, 'created key should keep the requested tag');

    const getApiKeyResponse = getApiKey(config, apiKey.id);
    expectStatus(getApiKeyResponse, 200, 'gets the api key');

    const getApiKeyBody = parseJson<ApiKeyRecord>(getApiKeyResponse);
    assert(getApiKeyBody.id === apiKey.id, 'fetched api key id did not match');

    const findApiKeysResponse = findApiKeys(config);
    expectStatus(findApiKeysResponse, 200, 'lists api keys');

    const apiKeyPage = parseJson<Page<ApiKeyRecord>>(findApiKeysResponse);
    assert(apiKeyPage.items.some((item) => item.id === apiKey.id), 'created api key should appear in the api key list');

    const introspectValidResponse = introspectApiKey(config, {
      token: apiKey.key,
      scope: 'keys:read',
      tags: [defaultTagName],
    });
    expectStatus(introspectValidResponse, 200, 'introspects a valid tagged api key');

    const introspectValidBody = parseJson<IntrospectApiKeyResponse>(introspectValidResponse);
    assert(introspectValidBody.valid === true, 'valid api key should introspect as valid');
    assert(introspectValidBody.key?.id === apiKey.id, 'introspection should return the created api key');

    const introspectInvalidTagResponse = introspectApiKey(config, {
      token: apiKey.key,
      scope: 'keys:read',
      tags: [customTagName],
    });
    expectStatus(introspectInvalidTagResponse, 200, 'rejects a valid key for the wrong tag');

    const invalidTagBody = parseJson<IntrospectApiKeyResponse>(introspectInvalidTagResponse);
    assert(invalidTagBody.valid === false, 'wrong tag should introspect as invalid');

    const deleteApiKeyResponse = deleteApiKey(config, apiKey.id);
    expectStatus(deleteApiKeyResponse, 204, 'deletes the api key');

    const postDeleteIntrospectResponse = introspectApiKey(config, {
      token: apiKey.key,
      scope: 'keys:read',
      tags: [defaultTagName],
    });
    expectStatus(postDeleteIntrospectResponse, 200, 'rejects an introspection after deletion');

    const postDeleteIntrospectBody = parseJson<IntrospectApiKeyResponse>(postDeleteIntrospectResponse);
    assert(postDeleteIntrospectBody.valid === false, 'deleted api key should introspect as invalid');

    safeDeleteTag(config, config.defaultNamespace, defaultTagName);
    safeDeleteTag(config, namespaceName, customTagName);
    safeDeleteNamespace(config, namespaceName);
  } catch (error) {
    safeDeleteTag(config, config.defaultNamespace, defaultTagName);
    safeDeleteTag(config, namespaceName, customTagName);
    safeDeleteNamespace(config, namespaceName);
    throw error;
  }
}
