# Exit code
0

# stdout
```
  ! Unused oxlint-disable directive (no problems were reported from FORWARD).
   ,-[files/test.vue:2:56]
 1 | <script setup lang="ts">
 2 | // oxlint-disable-next-line typescript/no-explicit-any FORWARD does not exist on Icon enum
   :                                                        ^^^^^^^
 3 | const icon = Yoco.Icon.FORWARD as any;
   `----

  ! Unused oxlint-disable directive (no problems were reported from does).
   ,-[files/test.vue:2:64]
 1 | <script setup lang="ts">
 2 | // oxlint-disable-next-line typescript/no-explicit-any FORWARD does not exist on Icon enum
   :                                                                ^^^^
 3 | const icon = Yoco.Icon.FORWARD as any;
   `----

  ! Unused oxlint-disable directive (no problems were reported from not).
   ,-[files/test.vue:2:69]
 1 | <script setup lang="ts">
 2 | // oxlint-disable-next-line typescript/no-explicit-any FORWARD does not exist on Icon enum
   :                                                                     ^^^
 3 | const icon = Yoco.Icon.FORWARD as any;
   `----

  ! Unused oxlint-disable directive (no problems were reported from exist).
   ,-[files/test.vue:2:73]
 1 | <script setup lang="ts">
 2 | // oxlint-disable-next-line typescript/no-explicit-any FORWARD does not exist on Icon enum
   :                                                                         ^^^^^
 3 | const icon = Yoco.Icon.FORWARD as any;
   `----

  ! Unused oxlint-disable directive (no problems were reported from on).
   ,-[files/test.vue:2:79]
 1 | <script setup lang="ts">
 2 | // oxlint-disable-next-line typescript/no-explicit-any FORWARD does not exist on Icon enum
   :                                                                               ^^
 3 | const icon = Yoco.Icon.FORWARD as any;
   `----

  ! Unused oxlint-disable directive (no problems were reported from Icon).
   ,-[files/test.vue:2:82]
 1 | <script setup lang="ts">
 2 | // oxlint-disable-next-line typescript/no-explicit-any FORWARD does not exist on Icon enum
   :                                                                                  ^^^^
 3 | const icon = Yoco.Icon.FORWARD as any;
   `----

  ! Unused oxlint-disable directive (no problems were reported from enum).
   ,-[files/test.vue:2:87]
 1 | <script setup lang="ts">
 2 | // oxlint-disable-next-line typescript/no-explicit-any FORWARD does not exist on Icon enum
   :                                                                                       ^^^^
 3 | const icon = Yoco.Icon.FORWARD as any;
   `----

Found 7 warnings and 0 errors.
Finished in Xms on 1 file with 98 rules using X threads.
```

# stderr
```
```
