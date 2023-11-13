import test from 'ava'

import { generatesHashingHex } from '../index.js'

test('test the hashing handler', (t) => {
  t.is(generatesHashingHex("key:value", false, false), "60dd8348")
})