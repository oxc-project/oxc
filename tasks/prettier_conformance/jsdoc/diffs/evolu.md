# JSDoc Diffs: evolu

Total JSDoc tags: 134
Files with diffs: 14

## Bugs Found

### BUG: "64-bit" reformatted to "64. Bit" (sentence boundary false positive)
In `packages/common/src/Type.ts`, the formatter treats the hyphen in "64-bit" as a sentence ending at "64." and capitalizes "Bit".

### BUG: `{@link EncryptedDbChange}` split across lines
In `packages/common/src/local-first/Protocol.ts`, a `{@link ...}` tag is broken across two lines:
```
+ * key. Returns an encrypted binary representation as {@link
+ * EncryptedDbChange}.
```

## Diff Categories

Most diffs (12 of 14 files) are line-wrapping differences where oxfmt re-wraps description text that contains `{@link ...}` tags differently than prettier-plugin-jsdoc. These are not bugs -- oxfmt breaks before `{@link Foo}` while prettier-plugin-jsdoc keeps the tag on the same line even if it slightly exceeds printWidth.

Other categories:
- **Markdown table alignment normalization** (Protocol.ts): removes leading colons from `| :--- |` to `| --- |`
- **List indent normalization** (Protocol.ts, Type.ts): adjusts continuation indent of nested list items

## `packages/common/src/Array.ts`
```diff
- * Checks if an array is non-empty and narrows its type to {@link NonEmptyArray}.
+ * Checks if an array is non-empty and narrows its type to
+ * {@link NonEmptyArray}.
```

## `packages/common/src/Redacted.ts`
```diff
- * Creates an {@link Eq} for {@link Redacted} values based on an equality function
- * for the underlying type.
+ * Creates an {@link Eq} for {@link Redacted} values based on an equality
+ * function for the underlying type.
```

## `packages/common/src/Result.ts`
```diff
- * Use {@link ok} to create a successful result and {@link err} to create an error
- * result.
+ * Use {@link ok} to create a successful result and {@link err} to create an
+ * error result.
```

## `packages/common/src/Store.ts`
```diff
- * A store for managing state with change notifications. Extends {@link Ref} with
- * subscriptions. Provides methods to get, set, and modify state, and to notify
- * listeners when the state changes.
+ * A store for managing state with change notifications. Extends {@link Ref}
+ * with subscriptions. Provides methods to get, set, and modify state, and to
+ * notify listeners when the state changes.
```

## `packages/common/src/Task.ts`
```diff
- * Task is called without a signal, it cannot be cancelled and {@link AbortError}
- * will never be returned. When called with a signal, the Task can be cancelled
- * and AbortError is added to the error union with precise type safety.
+ * Task is called without a signal, it cannot be cancelled and
+ * {@link AbortError} will never be returned. When called with a signal, the
+ * Task can be cancelled and AbortError is added to the error union with precise
+ * type safety.

-   * without a signal, {@link AbortError} cannot occur and the error type narrows
-   * accordingly.
+   * without a signal, {@link AbortError} cannot occur and the error type
+   * narrows accordingly.

- * This is a specialized version of a {@link Semaphore} with a permit count of 1.
+ * This is a specialized version of a {@link Semaphore} with a permit count of
+ * 1.
```

## `packages/common/src/Time.ts`
```diff
- * Creates a {@link Time} that returns a monotonically increasing number based on
- * a queueMicrotask.
+ * Creates a {@link Time} that returns a monotonically increasing number based
+ * on a queueMicrotask.

- * Accepts either a {@link DurationString} (e.g., "5m", "1h 30m") or milliseconds
- * as {@link NonNegativeInt}.
+ * Accepts either a {@link DurationString} (e.g., "5m", "1h 30m") or
+ * milliseconds as {@link NonNegativeInt}.
```

## `packages/common/src/Type.ts`
```diff
- * ObjectWithRecordType extends {@link Type} with additional `props` and `record`
- * properties for reflection.
+ * ObjectWithRecordType extends {@link Type} with additional `props` and
+ * `record` properties for reflection.

-   *         literal values (tagged unions).
+   *      literal values (tagged unions).
-   *         cases.
+   *      cases.

- * 64-bit signed integer.
+ * 64. Bit signed integer.

- * Converts each "nullable" property (a union that includes {@link Null}) into an
- * {@link optional} property. This means consumers can omit the property
+ * Converts each "nullable" property (a union that includes {@link Null}) into
+ * an {@link optional} property. This means consumers can omit the property
```

## `packages/common/src/local-first/Db.ts`
```diff
-   * configurations with proper URL formatting and {@link OwnerId} inclusion. The
-   * {@link OwnerId} in the URL enables relay authentication, allowing relay
+   * configurations with proper URL formatting and {@link OwnerId} inclusion.
+   * The {@link OwnerId} in the URL enables relay authentication, allowing relay
```

## `packages/common/src/local-first/Evolu.ts`
```diff
-   * Updates a row in the database and returns a {@link Result} with the existing
-   * {@link Id}.
+   * Updates a row in the database and returns a {@link Result} with the
+   * existing {@link Id}.

-   * Upserts a row in the database and returns a {@link Result} with the existing
-   * {@link Id}.
+   * Upserts a row in the database and returns a {@link Result} with the
+   * existing {@link Id}.

-/** Function returned by {@link Evolu#useOwner} to stop using an {@link SyncOwner}. */
+/**
+ * Function returned by {@link Evolu#useOwner} to stop using an
+ * {@link SyncOwner}.
+ */

- * Creates an {@link Evolu} instance for a platform configured with the specified
- * {@link EvoluSchema} and optional {@link EvoluConfig} providing a typed
- * interface for querying, mutating, and syncing your application's data.
+ * Creates an {@link Evolu} instance for a platform configured with the
+ * specified {@link EvoluSchema} and optional {@link EvoluConfig} providing a
+ * typed interface for querying, mutating, and syncing your application's data.
```

## `packages/common/src/local-first/Owner.ts`
```diff
- * assigned to an owner and encrypted with its {@link OwnerEncryptionKey}. Owners
- * allow partial sync, only the {@link AppOwner} is synced by default.
+ * assigned to an owner and encrypted with its {@link OwnerEncryptionKey}.
+ * Owners allow partial sync, only the {@link AppOwner} is synced by default.

- * Derives a {@link ShardOwner} from an {@link AppOwner} using the specified path.
+ * Derives a {@link ShardOwner} from an {@link AppOwner} using the specified
+ * path.

- * Read-only version of a {@link SharedOwner} for data sharing. Contains only the
- * {@link OwnerId} and {@link EncryptionKey} needed for others to read the shared
- * data without write access.
+ * Read-only version of a {@link SharedOwner} for data sharing. Contains only
+ * the {@link OwnerId} and {@link EncryptionKey} needed for others to read the
+ * shared data without write access.

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
- * | :----------------------------- | :------------------------ |
+ * | ------------------------------ | ------------------------- |

- *       containing **its own protocol version and the same `ownerId`**.
+ *     containing **its own protocol version and the same `ownerId`**.

- * key. Returns an encrypted binary representation as {@link EncryptedDbChange}.
+ * key. Returns an encrypted binary representation as {@link
+ * EncryptedDbChange}.
```

## `packages/common/src/local-first/Relay.ts`
```diff
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
- * - `updatedAt`: Set by Evolu on every row change, derived from {@link Timestamp}.
+ * - `updatedAt`: Set by Evolu on every row change, derived from
+ *   {@link Timestamp}.

- * Type Factory to create updateable {@link Type}. It makes everything except for
- * the `id` column optional (so they are not required) and ensures the
+ * Type Factory to create updateable {@link Type}. It makes everything except
+ * for the `id` column optional (so they are not required) and ensures the
```

## `packages/react/src/useQuery.ts`
```diff
- * {@link useQuery} blocks rendering until loading is completed. To avoid loading
- * waterfall with more queries, use {@link useQueries}.
+ * {@link useQuery} blocks rendering until loading is completed. To avoid
+ * loading waterfall with more queries, use {@link useQueries}.
```
