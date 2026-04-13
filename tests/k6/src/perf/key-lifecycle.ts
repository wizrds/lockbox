import { Options } from 'k6/options';

import {
  runCreateVerifyDeleteIteration,
  setupLifecycleFixture,
  teardownLifecycleFixture,
  LifecycleFixture,
} from '@/lib/tests.js';
import { getConfig } from '@/lib/utils.js';

export const options: Options = {
  vus: Number(__ENV.K6_VUS || 5),
  duration: __ENV.K6_DURATION || '30s',
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(95)<500'],
  },
};

const config = getConfig();

export function setup(): LifecycleFixture {
  return setupLifecycleFixture(config);
}

export default function (fixture: LifecycleFixture) {
  runCreateVerifyDeleteIteration(config, fixture, Number(__ENV.SLEEP_SECONDS || 0));
}

export function teardown(fixture: LifecycleFixture) {
  teardownLifecycleFixture(config, fixture);
}
