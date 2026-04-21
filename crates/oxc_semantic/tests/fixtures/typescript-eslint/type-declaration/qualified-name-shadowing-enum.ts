// Enum used in qualified name with shadowing type parameter

enum Status {
  Active = "active",
  Inactive = "inactive",
}

// Status.Active should reference the enum Status, not the type parameter
type Test<Status> = Status.Active;
