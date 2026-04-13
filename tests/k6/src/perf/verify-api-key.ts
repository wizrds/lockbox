import { Options } from 'k6/options';

import {
  runVerifyIteration,
  setupVerifyFixture,
  teardownVerifyFixture,
  VerifyFixture,
} from '@/lib/tests.js';
import { getConfig } from '@/lib/utils.js';

export const options: Options = {
  vus: Number(__ENV.K6_VUS || 20),
  duration: __ENV.K6_DURATION || '60s',
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(95)<250'],
  },
};

const config = getConfig();

export function setup(): VerifyFixture {
  return setupVerifyFixture(config);
}

export default function (fixture: VerifyFixture) {
  runVerifyIteration(config, fixture, Number(__ENV.SLEEP_SECONDS || 0));
}

export function teardown(fixture: VerifyFixture) {
  teardownVerifyFixture(config, fixture);
}
