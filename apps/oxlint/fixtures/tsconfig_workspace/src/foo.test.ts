// This test file should use src/tsconfig.test.json, not src/tsconfig.json
// The key is that it should be able to import from @test/* path alias
import { foo } from '@src/foo';
import { testHelper } from '@test/test-helper';

// This import should work because tsconfig.test.json includes test files
export function testFoo() {
    console.log(foo());
    console.log(testHelper());
}