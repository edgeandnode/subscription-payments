import {assert} from 'matchstick-as';

export function assertFields(
  name: string,
  id: string,
  expectedFields: Array<string>
): void {
  for (let i = 0; i < expectedFields.length; i += 2) {
    assert.fieldEquals(name, id, expectedFields[i], expectedFields[i + 1]);
  }
}
