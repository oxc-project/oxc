// DIVERGENCES.md#single-level-member-no-break
class A extends B {
  #sql;
  m() {
    foo.sql`DELETE FROM some_table_with_a_long_name WHERE some_column_name = ${someIdentifierName}`;
    this.sql`DELETE FROM some_table_with_a_long_name WHERE some_column_name = ${someIdentifierName}`;
    this.#sql`DELETE FROM some_table_with_a_long_name WHERE some_column_name = ${someIdentifierKey}`;
    call(foo.someVeryLongPropertyNameThatDoesNotFitOnTheLineEvenAfterBreakingXXXXXXXXXXXXXXXXXXXXXXXXXXX);
    call(this.someVeryLongPropertyNameThatDoesNotFitOnTheLineEvenAfterBreakingXXXXXXXXXXXXXXXXXXXXXXXXXX);
    call(super.someVeryLongPropertyNameThatDoesNotFitOnTheLineEvenAfterBreakingXXXXXXXXXXXXXXXXXXXXXXXXX);
    require.resolve("./a/long/long/long/long/long/long/long/long/long/long/long/long/long/path/to/module");

    // Not a single word, breaks like Prettier for now
    call(new.target.someVeryLongPropertyNameThatDoesNotFitOnTheLineEvenAfterBreakingXXXXXXXXXXXXXXXXXXXX);
    import.meta.resolve("./a/long/long/long/long/long/long/long/long/long/long/long/long/path/to/module");

    // A longer path still breaks into a list of properties
    call(this.someVeryLongPropertyNameXXXXXXXXXXXXXXXXXXXXXXXXXXXX.anotherVeryLongPropertyNameXXXXXXXXXXXXX);
  }
}
