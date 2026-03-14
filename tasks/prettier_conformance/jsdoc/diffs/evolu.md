# JSDoc Diffs: evolu

## `packages/common/src/Array.ts`

```diff
diff --git a/packages/common/src/Array.ts b/packages/common/src/Array.ts
index 9cc1e50..16145ad 100644
--- a/packages/common/src/Array.ts
+++ b/packages/common/src/Array.ts
@@ -99,7 +99,8 @@ export type NonEmptyArray<T> = [T, ...Array<T>];
 export type NonEmptyReadonlyArray<T> = readonly [T, ...ReadonlyArray<T>];
 
 /**
- * Checks if an array is non-empty and narrows its type to {@link NonEmptyArray}.
+ * Checks if an array is non-empty and narrows its type to
+ * {@link NonEmptyArray}.
  *
  * Use `if (!isNonEmptyArray(arr))` for empty checks.
  *
```

## `packages/common/src/Redacted.ts`

```diff
diff --git a/packages/common/src/Redacted.ts b/packages/common/src/Redacted.ts
index 86ca116..389b7de 100644
--- a/packages/common/src/Redacted.ts
+++ b/packages/common/src/Redacted.ts
@@ -98,8 +98,8 @@ export const isRedacted = (value: unknown): value is Redacted<unknown> =>
   Object.getPrototypeOf(value) === proto;
 
 /**
- * Creates an {@link Eq} for {@link Redacted} values based on an equality function
- * for the underlying type.
+ * Creates an {@link Eq} for {@link Redacted} values based on an equality
+ * function for the underlying type.
  *
  * ### Example
  *
```

## `packages/common/src/Result.ts`

```diff
diff --git a/packages/common/src/Result.ts b/packages/common/src/Result.ts
index 21e7738..010ddcd 100644
--- a/packages/common/src/Result.ts
+++ b/packages/common/src/Result.ts
@@ -12,8 +12,8 @@ import type { Task } from "./Task.js";
  * can have this too via the `Result` type.
  *
  * The `Result` type can be either {@link Ok} (success) or {@link Err} (error).
- * Use {@link ok} to create a successful result and {@link err} to create an error
- * result.
+ * Use {@link ok} to create a successful result and {@link err} to create an
+ * error result.
  *
  * Now let's look at how `Result` can be used for safe JSON parsing:
  *
```

## `packages/common/src/Store.ts`

```diff
diff --git a/packages/common/src/Store.ts b/packages/common/src/Store.ts
index d602b7f..0ec3fe3 100644
--- a/packages/common/src/Store.ts
+++ b/packages/common/src/Store.ts
@@ -2,9 +2,9 @@ import { Eq, eqStrict } from "./Eq.js";
 import { Ref } from "./Ref.js";
 
 /**
- * A store for managing state with change notifications. Extends {@link Ref} with
- * subscriptions. Provides methods to get, set, and modify state, and to notify
- * listeners when the state changes.
+ * A store for managing state with change notifications. Extends {@link Ref}
+ * with subscriptions. Provides methods to get, set, and modify state, and to
+ * notify listeners when the state changes.
  */
 export interface Store<T> extends Ref<T> {
   /**
```

## `packages/common/src/Task.ts`

```diff
diff --git a/packages/common/src/Task.ts b/packages/common/src/Task.ts
index 83e0c5c..8246250 100644
--- a/packages/common/src/Task.ts
+++ b/packages/common/src/Task.ts
@@ -13,9 +13,10 @@ import { NonNegativeInt, PositiveInt } from "./Type.js";
  * ### Cancellation
  *
  * Tasks support optional cancellation via signal in {@link TaskContext}. When a
- * Task is called without a signal, it cannot be cancelled and {@link AbortError}
- * will never be returned. When called with a signal, the Task can be cancelled
- * and AbortError is added to the error union with precise type safety.
+ * Task is called without a signal, it cannot be cancelled and
+ * {@link AbortError} will never be returned. When called with a signal, the
+ * Task can be cancelled and AbortError is added to the error union with precise
+ * type safety.
  *
  * When composing Tasks, we typically have context and want to abort ASAP by
  * passing it through. However, there are valid cases where we don't want to
@@ -147,8 +148,8 @@ export interface Task<T, E> {
    * Invoke the Task.
    *
    * Provide a context with an AbortSignal to enable cancellation. When called
-   * without a signal, {@link AbortError} cannot occur and the error type narrows
-   * accordingly.
+   * without a signal, {@link AbortError} cannot occur and the error type
+   * narrows accordingly.
    *
    * ### Example
    *
@@ -682,7 +683,8 @@ export const createSemaphore = (maxConcurrent: PositiveInt): Semaphore => {
 /**
  * A mutex (mutual exclusion) that ensures only one Task runs at a time.
  *
- * This is a specialized version of a {@link Semaphore} with a permit count of 1.
+ * This is a specialized version of a {@link Semaphore} with a permit count of
+ * 1.
  *
  * @see {@link createMutex} to create a mutex instance.
  */
```

## `packages/common/src/Time.ts`

```diff
diff --git a/packages/common/src/Time.ts b/packages/common/src/Time.ts
index bab423d..18dd11c 100644
--- a/packages/common/src/Time.ts
+++ b/packages/common/src/Time.ts
@@ -34,8 +34,8 @@ export const createTime = (): Time => {
 };
 
 /**
- * Creates a {@link Time} that returns a monotonically increasing number based on
- * a queueMicrotask.
+ * Creates a {@link Time} that returns a monotonically increasing number based
+ * on a queueMicrotask.
  */
 export const createTestTime = (): Time => {
   let now = 0;
@@ -121,8 +121,8 @@ export type Duration = DurationString | NonNegativeInt;
 /**
  * Converts a duration to milliseconds.
  *
- * Accepts either a {@link DurationString} (e.g., "5m", "1h 30m") or milliseconds
- * as {@link NonNegativeInt}.
+ * Accepts either a {@link DurationString} (e.g., "5m", "1h 30m") or
+ * milliseconds as {@link NonNegativeInt}.
  *
  * ### Example
  *
```

## `packages/common/src/Type.ts`

```diff
diff --git a/packages/common/src/Type.ts b/packages/common/src/Type.ts
index 97e7b8b..f61f568 100644
--- a/packages/common/src/Type.ts
+++ b/packages/common/src/Type.ts
@@ -3119,8 +3119,8 @@ export const formatObjectError = <Error extends TypeError>(
   });
 
 /**
- * ObjectWithRecordType extends {@link Type} with additional `props` and `record`
- * properties for reflection.
+ * ObjectWithRecordType extends {@link Type} with additional `props` and
+ * `record` properties for reflection.
  */
 export interface ObjectWithRecordType<
   Props extends Record<string, AnyType>,
@@ -3249,9 +3249,9 @@ export function union(...args: ReadonlyArray<any>): any {
    * 2. Enhance tagged union support:
    *
    *    - Detect if all arguments are objects with the same property but different
-   *         literal values (tagged unions).
+   *      literal values (tagged unions).
    *    - Generate a specialized function to improve validation performance for such
-   *         cases.
+   *      cases.
    */
 
   const members = args.map((arg) => (isType(arg) ? arg : literal(arg)));
@@ -3939,8 +3939,8 @@ export const partial = <Props extends Record<string, AnyType>>(
 };
 
 /**
- * Converts each “nullable” property (a union that includes {@link Null}) into an
- * {@link optional} property. This means consumers can omit the property
+ * Converts each “nullable” property (a union that includes {@link Null}) into
+ * an {@link optional} property. This means consumers can omit the property
  * entirely, or set it to `null`, or set it to the non-null member of the
  * union.
  *
```

## `packages/common/src/local-first/Db.ts`

```diff
diff --git a/packages/common/src/local-first/Db.ts b/packages/common/src/local-first/Db.ts
index 8b3fe8f..847dc14 100644
--- a/packages/common/src/local-first/Db.ts
+++ b/packages/common/src/local-first/Db.ts
@@ -114,8 +114,8 @@ export interface DbConfig extends ConsoleConfig, TimestampConfig {
    * {@link Evolu#useOwner}.
    *
    * Use {@link createOwnerWebSocketTransport} to create WebSocket transport
-   * configurations with proper URL formatting and {@link OwnerId} inclusion. The
-   * {@link OwnerId} in the URL enables relay authentication, allowing relay
+   * configurations with proper URL formatting and {@link OwnerId} inclusion.
+   * The {@link OwnerId} in the URL enables relay authentication, allowing relay
    * servers to control access (e.g., for paid tiers or private instances).
    *
    * The default value is:
```

## `packages/common/src/local-first/Evolu.ts`

```diff
diff --git a/packages/common/src/local-first/Evolu.ts b/packages/common/src/local-first/Evolu.ts
index ec43b5a..f28284a 100644
--- a/packages/common/src/local-first/Evolu.ts
+++ b/packages/common/src/local-first/Evolu.ts
@@ -280,8 +280,8 @@ export interface Evolu<S extends EvoluSchema = EvoluSchema> extends Disposable {
   insert: Mutation<S, "insert">;
 
   /**
-   * Updates a row in the database and returns a {@link Result} with the existing
-   * {@link Id}.
+   * Updates a row in the database and returns a {@link Result} with the
+   * existing {@link Id}.
    *
    * The first argument is the table name, and the second is an object
    * containing the row data including the required `id` field. An optional
@@ -329,8 +329,8 @@ export interface Evolu<S extends EvoluSchema = EvoluSchema> extends Disposable {
   update: Mutation<S, "update">;
 
   /**
-   * Upserts a row in the database and returns a {@link Result} with the existing
-   * {@link Id}.
+   * Upserts a row in the database and returns a {@link Result} with the
+   * existing {@link Id}.
    *
    * The first argument is the table name, and the second is an object
    * containing the row data including the required `id` field. An optional
@@ -454,7 +454,10 @@ export interface Evolu<S extends EvoluSchema = EvoluSchema> extends Disposable {
   readonly useOwner: (owner: SyncOwner) => UnuseOwner;
 }
 
-/** Function returned by {@link Evolu#useOwner} to stop using an {@link SyncOwner}. */
+/**
+ * Function returned by {@link Evolu#useOwner} to stop using an
+ * {@link SyncOwner}.
+ */
 export type UnuseOwner = () => void;
 
 /** Represents errors that can occur in Evolu. */
@@ -491,9 +494,9 @@ const evoluInstances = createInstances<SimpleName, InternalEvoluInstance>();
 let tabId: Id | null = null;
 
 /**
- * Creates an {@link Evolu} instance for a platform configured with the specified
- * {@link EvoluSchema} and optional {@link EvoluConfig} providing a typed
- * interface for querying, mutating, and syncing your application's data.
+ * Creates an {@link Evolu} instance for a platform configured with the
+ * specified {@link EvoluSchema} and optional {@link EvoluConfig} providing a
+ * typed interface for querying, mutating, and syncing your application's data.
  *
  * ### Example
  *
```

## `packages/common/src/local-first/Owner.ts`

```diff
diff --git a/packages/common/src/local-first/Owner.ts b/packages/common/src/local-first/Owner.ts
index 013cf1b..e719dba 100644
--- a/packages/common/src/local-first/Owner.ts
+++ b/packages/common/src/local-first/Owner.ts
@@ -34,8 +34,8 @@ export interface ReadonlyOwner {
 
 /**
  * The Owner represents ownership of data in Evolu. Every database change is
- * assigned to an owner and encrypted with its {@link OwnerEncryptionKey}. Owners
- * allow partial sync, only the {@link AppOwner} is synced by default.
+ * assigned to an owner and encrypted with its {@link OwnerEncryptionKey}.
+ * Owners allow partial sync, only the {@link AppOwner} is synced by default.
  *
  * Owners can also provide real data deletion, while individual changes in
  * local-first/distributed systems can only be soft deleted, entire owners can
@@ -227,7 +227,8 @@ export const createShardOwner = (secret: OwnerSecret): ShardOwner => {
 };
 
 /**
- * Derives a {@link ShardOwner} from an {@link AppOwner} using the specified path.
+ * Derives a {@link ShardOwner} from an {@link AppOwner} using the specified
+ * path.
  *
  * **Advantages of derived owners:**
  *
@@ -274,9 +275,9 @@ export const createSharedOwner = (secret: OwnerSecret): SharedOwner => ({
 });
 
 /**
- * Read-only version of a {@link SharedOwner} for data sharing. Contains only the
- * {@link OwnerId} and {@link EncryptionKey} needed for others to read the shared
- * data without write access.
+ * Read-only version of a {@link SharedOwner} for data sharing. Contains only
+ * the {@link OwnerId} and {@link EncryptionKey} needed for others to read the
+ * shared data without write access.
  */
 export interface SharedReadonlyOwner extends ReadonlyOwner {
   readonly type: "SharedReadonlyOwner";
@@ -303,10 +304,11 @@ export type OwnerTransport = OwnerWebSocketTransport;
  *
  * ### Authentication via URL
  *
- * The {@link OwnerId} is passed as a URL query parameter. While this approach is
- * generally discouraged for authentication tokens (they get logged), it's safe
- * here because OwnerId is pseudonymous and used only for access verification -
- * it provides no ability to read encrypted data or write changes.
+ * The {@link OwnerId} is passed as a URL query parameter. While this approach
+ * is generally discouraged for authentication tokens (they get logged), it's
+ * safe here because OwnerId is pseudonymous and used only for access
+ * verification - it provides no ability to read encrypted data or write
+ * changes.
  *
  * See: [HTTP headers in Websockets client
  * API](https://stackoverflow.com/questions/4361173/http-headers-in-websockets-client-api/74564827#74564827)
```

## `packages/common/src/local-first/Protocol.ts`

```diff
diff --git a/packages/common/src/local-first/Protocol.ts b/packages/common/src/local-first/Protocol.ts
index e4ba051..df98b52 100644
--- a/packages/common/src/local-first/Protocol.ts
+++ b/packages/common/src/local-first/Protocol.ts
@@ -141,7 +141,7 @@
  *
  *   - If the versions match, synchronization proceeds as normal.
  *   - If the versions do not match, the non-initiator responds with a message
- *       containing **its own protocol version and the same `ownerId`**.
+ *     containing **its own protocol version and the same `ownerId`**.
  * - The initiator can then detect the version mismatch for that specific owner
  *   and handle it appropriately (e.g., prompt for an update or halt sync).
  *
@@ -1708,7 +1708,8 @@ export const decodeFlags = (
 
 /**
  * Encodes and encrypts a {@link DbChange} using the provided owner's encryption
- * key. Returns an encrypted binary representation as {@link EncryptedDbChange}.
+ * key. Returns an encrypted binary representation as
+ * {@link EncryptedDbChange}.
  *
  * The format includes the protocol version for backward compatibility and the
  * timestamp for tamper-proof verification that the timestamp matches the change
```

## `packages/common/src/local-first/Relay.ts`

```diff
diff --git a/packages/common/src/local-first/Relay.ts b/packages/common/src/local-first/Relay.ts
index bb2b67a..373415f 100644
--- a/packages/common/src/local-first/Relay.ts
+++ b/packages/common/src/local-first/Relay.ts
@@ -120,10 +120,10 @@ export const createRelaySqliteStorage =
        * Lazily authorizes the initiator's {@link OwnerWriteKey} for the given
        * {@link OwnerId}.
        *
-       * - If the {@link OwnerId} does not exist, it is created and associated with
-       *   the provided write key.
-       * - If the {@link OwnerId} exists, the provided write key is compared to the
-       *   stored key.
+       * - If the {@link OwnerId} does not exist, it is created and associated
+       *   with the provided write key.
+       * - If the {@link OwnerId} exists, the provided write key is compared to
+       *   the stored key.
        */
       validateWriteKey: (ownerId, writeKey) => {
         const selectWriteKey = deps.sqlite.exec<{ writeKey: OwnerWriteKey }>(
```

## `packages/common/src/local-first/Schema.ts`

```diff
diff --git a/packages/common/src/local-first/Schema.ts b/packages/common/src/local-first/Schema.ts
index f510635..cc9ca0d 100644
--- a/packages/common/src/local-first/Schema.ts
+++ b/packages/common/src/local-first/Schema.ts
@@ -231,7 +231,8 @@ export type CreateQuery<S extends EvoluSchema> = <R extends Row>(
  * System columns that are implicitly defined by Evolu.
  *
  * - `createdAt`: Set by Evolu on row creation, derived from {@link Timestamp}.
- * - `updatedAt`: Set by Evolu on every row change, derived from {@link Timestamp}.
+ * - `updatedAt`: Set by Evolu on every row change, derived from
+ *   {@link Timestamp}.
  * - `isDeleted`: Soft delete flag created by Evolu and used by the developer to
  *   mark rows as deleted.
  * - `ownerId`: Represents ownership and logically partitions the database.
@@ -362,8 +363,8 @@ export type Insertable<Props extends Record<string, AnyType>> = InferInput<
 >;
 
 /**
- * Type Factory to create updateable {@link Type}. It makes everything except for
- * the `id` column optional (so they are not required) and ensures the
+ * Type Factory to create updateable {@link Type}. It makes everything except
+ * for the `id` column optional (so they are not required) and ensures the
  * {@link maxMutationSize}.
  *
  * ### Example
```

## `packages/react/src/useQuery.ts`

```diff
diff --git a/packages/react/src/useQuery.ts b/packages/react/src/useQuery.ts
index 0695125..163eb7f 100644
--- a/packages/react/src/useQuery.ts
+++ b/packages/react/src/useQuery.ts
@@ -10,8 +10,8 @@ import { useIsSsr } from "./useIsSsr.js";
  * properties that are automatically updated when data changes.
  *
  * Note that {@link useQuery} uses React Suspense. It means every usage of
- * {@link useQuery} blocks rendering until loading is completed. To avoid loading
- * waterfall with more queries, use {@link useQueries}.
+ * {@link useQuery} blocks rendering until loading is completed. To avoid
+ * loading waterfall with more queries, use {@link useQueries}.
  *
  * The `promise` option allows preloading queries before rendering, which can be
  * useful for complex queries that might take noticeable time even with local
```

