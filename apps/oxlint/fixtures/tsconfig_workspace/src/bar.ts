// This regular source file should use src/tsconfig.json
// It should NOT be able to import from @test/* because that's only in tsconfig.test.json
import { foo } from '@src/foo';

// This import should fail if the correct tsconfig is used
import { testHelper } from '@test/test-helper';

export function bar() {
    return foo() + " bar";
}