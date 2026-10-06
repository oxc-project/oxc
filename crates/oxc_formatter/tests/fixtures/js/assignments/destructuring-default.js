// Nested object patterns with `= {}` defaults: break shapes
// issue #16089
const { data: { fooBarBazProfiles = [] } = {}, isLoading: isFooBarLoading } = useFooBarBazQuxProfiles(
  profileId,
  profileQueryParams
);
// issue #16127
const { assignmentPattern: { nestedAssignmentPattern = {} } = {} } = call();
// issue #16520
const {
  data: { args: { something } } = {
    args: { something: []},
  }
} = obj;
