import { Options } from 'k6/options';

import { runSmokeFlow } from '@/lib/tests.js';
import { getConfig } from '@/lib/utils.js';

export const options: Options = {
  vus: 1,
  iterations: 1,
};

const config = getConfig();

export default function () {
  runSmokeFlow(config);
}
