# JSDoc Diffs: evolu (round 6, 2026-03-17)

13 files with diffs, 55 JSDoc tags

## `packages/common/src/Array.ts`

```diff
@@ -99,7 +99,8 @@ export type NonEmptyReadonlyArray<T> = readonly [T, ...ReadonlyArray<T>];

 /**
- * Checks if an array is non-empty and narrows its type to {@link NonEmptyArray}.
+ * Checks if an array is non-empty and narrows its type to
+ * {@link NonEmptyArray}.
```

## `packages/common/src/Redacted.ts`

```diff
@@ -98,8 +98,8 @@ export const isRedacted = (value: unknown): value is Redacted<unknown> =>

 /**
- * Creates an {@link Eq} for {@link Redacted} values based on an equality function
- * for the underlying type.
+ * Creates an {@link Eq} for {@link Redacted} values based on an equality
+ * function for the underlying type.
```

## `packages/common/src/Result.ts`

```diff
@@ -12,8 +12,8 @@ import type { Task } from "./Task.js";
  * The `Result` type can be either {@link Ok} (success) or {@link Err} (error).
- * Use {@link ok} to create a successful result and {@link err} to create an error
- * result.
+ * Use {@link ok} to create a successful result and {@link err} to create an
+ * error result.
```

## `packages/common/src/Task.ts`

```diff
@@ -13,9 +13,10 @@ import { NonNegativeInt, PositiveInt } from "./Type.js";
  * Tasks support optional cancellation via signal in {@link TaskContext}. When a
- * Task is called without a signal, it cannot be cancelled and {@link AbortError}
- * will never be returned. When called with a signal, the Task can be cancelled
- * and AbortError is added to the error union with precise type safety.
+ * Task is called without a signal, it cannot be cancelled and
+ * {@link AbortError} will never be returned. When called with a signal, the
+ * Task can be cancelled and AbortError is added to the error union with precise
+ * type safety.

@@ -147,8 +148,8 @@ export interface Task<T, E> {
    * Provide a context with an AbortSignal to enable cancellation. When called
-   * without a signal, {@link AbortError} cannot occur and the error type narrows
-   * accordingly.
+   * without a signal, {@link AbortError} cannot occur and the error type
+   * narrows accordingly.
```

## `packages/common/src/Time.ts`

```diff
@@ -121,8 +121,8 @@ export type Duration = DurationString | NonNegativeInt;
- * Accepts either a {@link DurationString} (e.g., "5m", "1h 30m") or milliseconds
- * as {@link NonNegativeInt}.
+ * Accepts either a {@link DurationString} (e.g., "5m", "1h 30m") or
+ * milliseconds as {@link NonNegativeInt}.
```

## `packages/common/src/Type.ts`

```diff
@@ -3249,9 +3249,9 @@ export function union(...args: ReadonlyArray<any>): any {
   *    - Detect if all arguments are objects with the same property but different
-  *         literal values (tagged unions).
+  *      literal values (tagged unions).
   *    - Generate a specialized function to improve validation performance for such
-  *         cases.
+  *      cases.
```

## `packages/common/src/local-first/Db.ts`

```diff
@@ -114,8 +114,8 @@ export interface DbConfig extends ConsoleConfig, TimestampConfig {
    * Use {@link createOwnerWebSocketTransport} to create WebSocket transport
-   * configurations with proper URL formatting and {@link OwnerId} inclusion. The
-   * {@link OwnerId} in the URL enables relay authentication, allowing relay
+   * configurations with proper URL formatting and {@link OwnerId} inclusion.
+   * The {@link OwnerId} in the URL enables relay authentication, allowing relay
```

## `packages/common/src/local-first/Evolu.ts`

```diff
@@ -280,8 +280,8 @@ export interface Evolu<S extends EvoluSchema = EvoluSchema> extends Disposable {
-   * Updates a row in the database and returns a {@link Result} with the existing
-   * {@link Id}.
+   * Updates a row in the database and returns a {@link Result} with the
+   * existing {@link Id}.

@@ -329,8 +329,8 @@ export interface Evolu<S extends EvoluSchema = EvoluSchema> extends Disposable {
-   * Upserts a row in the database and returns a {@link Result} with the existing
-   * {@link Id}.
+   * Upserts a row in the database and returns a {@link Result} with the
+   * existing {@link Id}.

@@ -491,9 +491,9 @@ let tabId: Id | null = null;
- * Creates an {@link Evolu} instance for a platform configured with the specified
- * {@link EvoluSchema} and optional {@link EvoluConfig} providing a typed
- * interface for querying, mutating, and syncing your application's data.
+ * Creates an {@link Evolu} instance for a platform configured with the
+ * specified {@link EvoluSchema} and optional {@link EvoluConfig} providing a
+ * typed interface for querying, mutating, and syncing your application's data.
```

## `packages/common/src/local-first/Owner.ts`

```diff
@@ -34,8 +34,8 @@ export interface ReadonlyOwner {
- * assigned to an owner and encrypted with its {@link OwnerEncryptionKey}. Owners
- * allow partial sync, only the {@link AppOwner} is synced by default.
+ * assigned to an owner and encrypted with its {@link OwnerEncryptionKey}.
+ * Owners allow partial sync, only the {@link AppOwner} is synced by default.

@@ -211,8 +211,7 @@ export const createAppOwner = (secret: OwnerSecret): AppOwner => ({
  * Can be created from {@link OwnerSecret} via {@link createShardOwner} or
- * deterministically derived from {@link AppOwner} using
- * {@link deriveShardOwner}.
+ * deterministically derived from {@link AppOwner} using {@link deriveShardOwner}.

@@ -227,7 +226,8 @@ export const createShardOwner = (secret: OwnerSecret): ShardOwner => {
- * Derives a {@link ShardOwner} from an {@link AppOwner} using the specified path.
+ * Derives a {@link ShardOwner} from an {@link AppOwner} using the specified
+ * path.

@@ -303,10 +303,11 @@ export type OwnerTransport = OwnerWebSocketTransport;
- * The {@link OwnerId} is passed as a URL query parameter. While this approach is
- * generally discouraged for authentication tokens (they get logged), it's safe
- * here because OwnerId is pseudonymous and used only for access verification -
- * it provides no ability to read encrypted data or write changes.
+ * The {@link OwnerId} is passed as a URL query parameter. While this approach
+ * is generally discouraged for authentication tokens (they get logged), it's
+ * safe here because OwnerId is pseudonymous and used only for access
+ * verification - it provides no ability to read encrypted data or write
+ * changes.
```

## `packages/common/src/local-first/Protocol.ts`

```diff
@@ -141,7 +141,7 @@
  *   - If the versions do not match, the non-initiator responds with a message
- *       containing **its own protocol version and the same `ownerId`**.
+ *     containing **its own protocol version and the same `ownerId`**.

@@ -1708,7 +1708,8 @@ export const decodeFlags = (
  * Encodes and encrypts a {@link DbChange} using the provided owner's encryption
- * key. Returns an encrypted binary representation as {@link EncryptedDbChange}.
+ * key. Returns an encrypted binary representation as
+ * {@link EncryptedDbChange}.
```

## `packages/common/src/local-first/Relay.ts`

```diff
@@ -120,10 +120,10 @@ export const createRelaySqliteStorage =
-       * - If the {@link OwnerId} does not exist, it is created and associated with
-       *   the provided write key.
-       * - If the {@link OwnerId} exists, the provided write key is compared to the
-       *   stored key.
+       * - If the {@link OwnerId} does not exist, it is created and associated
+       *   with the provided write key.
+       * - If the {@link OwnerId} exists, the provided write key is compared to
+       *   the stored key.
```

## `packages/common/src/local-first/Schema.ts`

```diff
@@ -231,7 +231,8 @@ export type CreateQuery<S extends EvoluSchema> = <R extends Row>(
  * - `createdAt`: Set by Evolu on row creation, derived from {@link Timestamp}.
- * - `updatedAt`: Set by Evolu on every row change, derived from {@link Timestamp}.
+ * - `updatedAt`: Set by Evolu on every row change, derived from
+ *   {@link Timestamp}.

@@ -362,8 +363,8 @@ export type Insertable<Props extends Record<string, AnyType>> = InferInput<
- * Type Factory to create updateable {@link Type}. It makes everything except for
- * the `id` column optional (so they are not required) and ensures the
+ * Type Factory to create updateable {@link Type}. It makes everything except
+ * for the `id` column optional (so they are not required) and ensures the
```

## `packages/react/src/useQuery.ts`

```diff
@@ -10,8 +10,8 @@ import { useIsSsr } from "./useIsSsr.js";
  * Note that {@link useQuery} uses React Suspense. It means every usage of
- * {@link useQuery} blocks rendering until loading is completed. To avoid loading
- * waterfall with more queries, use {@link useQueries}.
+ * {@link useQuery} blocks rendering until loading is completed. To avoid
+ * loading waterfall with more queries, use {@link useQueries}.
```
