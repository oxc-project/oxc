// This test file should use src/tsconfig.test.json, not src/tsconfig.json
// The key is that it should be able to import from @test/* path alias
import { foo } from '@src/foo';
import { testHelper } from '@test/test-helper';

describe('foo tests', () => {
    it('should work', () => {
        expect(foo()).toBe('foo');
        expect(testHelper()).toBe('helper');
    });
});